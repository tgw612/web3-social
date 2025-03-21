use crate::blockchain::ethereum::EthClient;
use crate::blockchain::solana::SolanaClient;
use crate::models::asset::{Asset, AssetType, TokenBalance, NftAsset};
use crate::utils::error::ServiceError;
use ethers::prelude::*;
use diesel::prelude::*;
use diesel::pg::PgConnection;
use diesel::{BoolExpressionMethods, ExpressionMethods, QueryDsl, RunQueryDsl, OptionalExtension};
use std::sync::Arc;
use std::sync::Mutex;
use redis::Client as RedisClient;

/// 资产服务，处理多链资产聚合和展示
pub struct AssetService {
    db: Arc<Mutex<PgConnection>>,
    redis: Arc<RedisClient>,
    eth_client: Arc<EthClient>,
    solana_client: Arc<SolanaClient>,
}

impl AssetService {
    pub fn new(
        db: Arc<Mutex<PgConnection>>,
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
        use crate::schema::assets::dsl::*;
        let conn = self.db.lock().unwrap();
        let result = assets
            .filter(wallet_address.eq(wallet_address))
            .order(value_usd.desc())
            .load::<Asset>(&*conn)
            .map_err(|_| ServiceError::InternalServerError)?;

        Ok(result)
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
        use crate::schema::assets::dsl::*;
        let conn = self.db.lock().unwrap();
        diesel::insert_into(assets)
            .values((
                wallet_address.eq(wallet_address),
                chain_id.eq(&asset.chain_id),
                asset_type.eq(&asset.asset_type),
                symbol.eq(&asset.symbol),
                name.eq(&asset.name),
                contract_address.eq(&asset.contract_address),
                balance.eq(asset.balance),
                decimals.eq(asset.decimals),
                price_usd.eq(asset.price_usd),
                value_usd.eq(asset.value_usd)
            ))
            .on_conflict((wallet_address, chain_id, contract_address))
            .do_update()
            .set((
                balance.eq(asset.balance),
                price_usd.eq(asset.price_usd),
                value_usd.eq(asset.value_usd),
                updated_at.eq(diesel::dsl::now)
            ))
            .execute(&*conn)
            .map_err(|_| ServiceError::InternalServerError)?;

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
        use crate::schema::nft_assets::dsl::*;
        let conn = self.db.lock().unwrap();
        let nfts = nft_assets
            .filter(wallet_address.eq(wallet_address))
            .load::<NftAsset>(&*conn)
            .map_err(|_| ServiceError::InternalServerError)?;

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
        use crate::schema::nft_assets::dsl::*;
        let conn = self.db.lock().unwrap();
        diesel::insert_into(nft_assets)
            .values((
                wallet_address.eq(wallet_address),
                chain_id.eq(&nft.chain_id),
                contract_address.eq(&nft.contract_address),
                token_id.eq(&nft.token_id),
                name.eq(&nft.name),
                image_url.eq(&nft.image_url),
                metadata_url.eq(&nft.metadata_url)
            ))
            .on_conflict((wallet_address, chain_id, contract_address, token_id))
            .do_update()
            .set((
                name.eq(&nft.name),
                image_url.eq(&nft.image_url),
                metadata_url.eq(&nft.metadata_url),
                updated_at.eq(diesel::dsl::now)
            ))
            .execute(&*conn)
            .map_err(|_| ServiceError::InternalServerError)?;

        Ok(())
    }

    /// 获取用户资产总价值（美元）
    pub async fn get_total_value(&self, wallet_address: &str) -> Result<f64, ServiceError> {
        use crate::schema::assets::dsl::*;
        let conn = self.db.lock().unwrap();
        let total: f64 = assets
            .filter(wallet_address.eq(wallet_address))
            .select(diesel::dsl::sum(value_usd))
            .first(&*conn)
            .unwrap_or(0.0);

        Ok(total)
    }
}