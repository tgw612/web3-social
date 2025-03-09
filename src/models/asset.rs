use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Asset {
    pub chain: String,           // ETH, SOL, etc.
    pub token_address: String,   // 代币合约地址，原生代币为空字符串
    pub symbol: String,          // 代币符号
    pub name: String,            // 代币名称
    pub decimals: u8,            // 代币精度
    pub balance: String,         // 原始余额（字符串格式，避免精度问题）
    pub balance_usd: f64,        // USD价值
    pub price_usd: f64,          // 单价（USD）
    pub token_type: String,      // NATIVE, ERC20, SPL
    pub logo_url: Option<String>, // 代币Logo URL
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NFT {
    pub chain: String,           // ETH, SOL, etc.
    pub contract_address: String, // NFT合约地址
    pub token_id: String,        // Token ID
    pub name: String,            // NFT名称
    pub collection_name: Option<String>, // 集合名称
    pub description: Option<String>, // 描述
    pub image_url: Option<String>, // 图片URL
    pub metadata_url: Option<String>, // 元数据URL
    pub floor_price_usd: Option<f64>, // 地板价（USD）
    pub token_type: String,      // ERC721, ERC1155, Metaplex, etc.
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenBalance {
    pub chain: String,
    pub address: String,
    pub raw_balance: String,
    pub decimals: u8,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AssetsResponse {
    pub tokens: Vec<Asset>,
    pub nfts: Vec<NFT>,
    pub total_value_usd: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AssetsQueryParams {
    pub wallet_address: String,
    pub wallet_chain: Option<String>, // 不指定时，查询所有支持的链
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenPrice {
    pub symbol: String,
    pub name: String,
    pub price_usd: f64,
    pub change_24h: Option<f64>,
    pub market_cap_usd: Option<f64>,
    pub updated_at: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TransactionVerification {
    pub is_valid: bool,
    pub transaction_hash: String,
    pub from_address: String,
    pub to_address: String,
    pub value: String,
    pub token_address: Option<String>,
    pub token_symbol: Option<String>,
    pub timestamp: i64,
    pub block_number: u64,
    pub status: String, // "success", "pending", "failed"
    pub chain: String,
} 