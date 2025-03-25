use crate::models::asset::*;
use crate::models::rbatis_entities::{AssetEntity, NftAssetEntity};
use crate::utils::error::ServiceError;
use ethers::prelude::*;
use std::sync::Arc;
use redis::Client as RedisClient;
use rbatis::RBatis;
/// 资产服务，处理多链资产聚合和展示
pub struct AssetService {
    db: Arc<RBatis>,
    redis: Arc<RedisClient>,
    eth_client: Arc<EthClient>,
    solana_client: Arc<SolanaClient>,
}

impl AssetService {
    pub fn new(
        db: Arc<RBatis>,
        redis: Arc<RedisClient>,
        eth_client: Arc<EthClient>,
        solana_client: Arc<SolanaClient>,
    ) -> Self {
        Self {
            db,
            redis,
            eth_client,
            solana_client,
        }
    }

    /// 获取用户资产总览
    pub async fn get_user_assets(&self, wallet_address: &str) -> Result<Vec<Asset>, ServiceError> {
        // 获取用户在数据库中已有的资产记录
        let cached_assets = self.get_cached_assets(wallet_address).await?;

        // 如果缓存不存在或已过期，则从链上重新获取
        if cached_assets.is_empty() {
            // 创建或更新资产记录
            self.update_assets(wallet_address).await?;
            // 再次获取资产
            self.get_cached_assets(wallet_address).await
        } else {
            Ok(cached_assets)
        }
    }

    /// 从数据库获取已缓存的资产
    async fn get_cached_assets(&self, wallet_address: &str) -> Result<Vec<Asset>, ServiceError> {
        let entities = self.db.query_list_by_column::<AssetEntity>("wallet_address", &wallet_address).await
            .map_err(|_| ServiceError::InternalServerError)?;
            
        // 将AssetEntity转换为Asset
        let assets: Vec<Asset> = entities.into_iter().map(|e| Asset {
            symbol: e.symbol,
            name: e.name,
            balance: e.balance,
            decimals: e.decimals.map(|d| d as u8),
            price_usd: e.price_usd,
            value_usd: e.value_usd,
            chain_id: e.chain_id,
            asset_type: e.asset_type.clone(),
            contract_address: e.contract_address.clone(),
            created_at: e.created_at.clone(),
            updated_at: e.updated_at.clone(),
        }).collect();

        Ok(assets)
    }

    /// 更新用户资产数据
    async fn update_assets(&self, wallet_address: &str) -> Result<(), ServiceError> {
        // 获取ETH链上资产
        let eth_assets = self.eth_client.get_assets(wallet_address).await?;
        
        // 获取Solana链上资产
        let solana_assets = self.solana_client.get_assets(wallet_address).await?;
        
        // 合并所有资产
        let mut all_assets = Vec::with_capacity(eth_assets.len() + solana_assets.len());
        all_assets.extend(eth_assets);
        all_assets.extend(solana_assets);

        // 获取最新资产价格并计算价值
        self.update_asset_prices(&mut all_assets).await?;
        
        // 保存到数据库
        for asset in all_assets {
            self.save_asset(wallet_address, &asset).await?;
        }

        Ok(())
    }

    /// 更新资产价格
    async fn update_asset_prices(&self, assets: &mut Vec<Asset>) -> Result<(), ServiceError> {
        // 获取所有代币地址
        let token_addresses: Vec<String> = assets
            .iter()
            .filter(|a| a.asset_type == AssetType::Token)
            .map(|a| a.contract_address.clone().unwrap_or_default())
            .collect();

        // 从外部API获取价格（模拟实现）
        let prices = self.fetch_token_prices(&token_addresses).await?;
        
        // 更新资产价格
        for asset in assets.iter_mut() {
            if asset.asset_type == AssetType::Token {
                if let Some(addr) = &asset.contract_address {
                    if let Some(price) = prices.get(addr) {
                        asset.price_usd = Some(*price);
                        // 计算总价值
                        if let Some(balance) = asset.balance {
                            asset.value_usd = Some(balance * *price);
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// 从外部API获取代币价格
    async fn fetch_token_prices(&self, token_addresses: &[String]) -> Result<std::collections::HashMap<String, f64>, ServiceError> {
        let mut prices = std::collections::HashMap::new();
        
        for addr in token_addresses {
            let cache_key = format!("token_price:{}", addr);
            let mut con = self.redis.get_async_connection().await
                .map_err(|_| ServiceError::InternalServerError)?;
            
            let cached_price: Option<f64> = redis::cmd("GET")
                .arg(&cache_key)
                .query_async(&mut con)
                .await
                .map_err(|_| ServiceError::InternalServerError)?;
                
            if let Some(price) = cached_price {
                prices.insert(addr.clone(), price);
            } else {
                let price = 1.0; // 模拟价格
                prices.insert(addr.clone(), price);
                
                redis::cmd("SETEX")
                    .arg(&cache_key)
                    .arg(300)
                    .arg(price)
                    .query_async(&mut con)
                    .await
                    .map_err(|_| ServiceError::InternalServerError)?;
            }
        }
        
        Ok(prices)
    }

    /// 保存资产到数据库
    async fn save_asset(&self, wallet_address: &str, asset: &Asset) -> Result<(), ServiceError> {
        // 检查资产是否已存在
        let existing = self.db.query_by_column::<Option<AssetEntity>>("wallet_address", &wallet_address)
            .await
            .map_err(|_| ServiceError::InternalServerError)?;
            
        // 创建AssetEntity
        let entity = AssetEntity {
            wallet_address: wallet_address.to_string(),
            chain_id: asset.chain_id,
            asset_type: asset.asset_type.clone(),
            symbol: asset.symbol.clone(),
            name: asset.name.clone(),
            contract_address: asset.contract_address.clone(),
            balance: asset.balance,
            decimals: asset.decimals.map(|d| d as i32),
            price_usd: asset.price_usd,
            value_usd: asset.value_usd,
            created_at: None, // 数据库会自动设置
            updated_at: None, // 数据库会自动设置
        };
        
        if existing.is_some() {
            // 更新现有记录
            self.db.update_by_column::<AssetEntity>(&entity, "wallet_address")
                .await
                .map_err(|_| ServiceError::InternalServerError)?;
        } else {
            // 插入新记录
            self.db.save(&entity, &[])
                .await
                .map_err(|_| ServiceError::InternalServerError)?;
        }

        Ok(())
    }

    /// 获取特定代币余额
    pub async fn get_token_balance(&self, wallet_address: &str, chain_id: i32, contract_address: &str) -> Result<TokenBalance, ServiceError> {
        // 根据链类型选择不同的客户端
        let balance = match chain_id {
            1 => self.eth_client.get_token_balance(wallet_address, contract_address).await?,
            2 => self.solana_client.get_token_balance(wallet_address, contract_address).await?,
            _ => return Err(ServiceError::BadRequest("不支持的链ID".into())),
        };

        Ok(balance)
    }

    /// 获取用户NFT资产
    pub async fn get_user_nfts(&self, wallet_address: &str) -> Result<Vec<NftAsset>, ServiceError> {
        let entities = self.db.query_list_by_column::<NftAssetEntity>("wallet_address", &wallet_address)
            .await
            .map_err(|_| ServiceError::InternalServerError)?;
            
        // 将NftAssetEntity转换为NftAsset
        let nfts: Vec<NftAsset> = entities.into_iter().map(|e| NftAsset {
            chain_id: e.chain_id,
            contract_address: e.contract_address,
            token_id: e.token_id,
            name: e.name,
            image_url: e.image_url,
            metadata_url: e.metadata_url,
            created_at: e.created_at.map(Into::into),
            updated_at: e.updated_at.map(Into::into),
        }).collect();

        if nfts.is_empty() {
            // 获取以太坊NFT
            let eth_nfts = self.eth_client.get_nfts(wallet_address).await?;
            
            // 获取Solana NFT
            let solana_nfts = self.solana_client.get_nfts(wallet_address).await?;
            
            // 合并所有NFT
            let mut all_nfts = Vec::with_capacity(eth_nfts.len() + solana_nfts.len());
            all_nfts.extend(eth_nfts);
            all_nfts.extend(solana_nfts);
            
            // 保存到数据库
            for nft in &all_nfts {
                self.save_nft(wallet_address, nft).await?;
            }
            
            Ok(all_nfts)
        } else {
            Ok(nfts)
        }
    }

    /// 保存NFT到数据库
    async fn save_nft(&self, wallet_address: &str, nft: &NftAsset) -> Result<(), ServiceError> {
        // 检查NFT是否已存在
        let existing = self.db.query_by_column::<Option<NftAssetEntity>>("wallet_address", &wallet_address)
            .await
            .map_err(|_| ServiceError::InternalServerError)?;
            
        // 创建NftAssetEntity
        let entity = NftAssetEntity {
            wallet_address: wallet_address.to_string(),
            chain_id: nft.chain_id,
            contract_address: nft.contract_address.clone(),
            token_id: nft.token_id.clone(),
            name: nft.name.clone(),
            image_url: nft.image_url.clone(),
            metadata_url: nft.metadata_url.clone(),
            created_at: None, // 数据库会自动设置
            updated_at: None, // 数据库会自动设置
        };
        
        if existing.is_some() {
            // 更新现有记录
            self.db.update_by_column::<NftAssetEntity>(&entity, "wallet_address")
                .await
                .map_err(|_| ServiceError::InternalServerError)?;
        } else {
            // 插入新记录
            self.db.save(&entity, &[])
                .await
                .map_err(|_| ServiceError::InternalServerError)?;
        }

        Ok(())
    }

    /// 获取用户资产总价值（美元）
    pub async fn get_total_value(&self, wallet_address: &str) -> Result<f64, ServiceError> {
        // 使用rbatis执行原生SQL查询获取总价值
        let sql = "SELECT COALESCE(SUM(value_usd), 0) FROM assets WHERE wallet_address = ?".to_string();
        
        let total: Option<f64> = self.db.query_decode(&sql, vec![rbs::to_value(wallet_address)])
            .await
            .map_err(|e| {
                log::error!("获取资产总价值失败: {}", e);
                ServiceError::InternalServerError
            })?;
        
        // 处理空值并保留两位小数
        let total = total.unwrap_or(0.0);
        let total = (total * 100.0).round() / 100.0;
        Ok(total)
    }
    
}