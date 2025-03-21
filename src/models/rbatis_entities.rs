use serde::{Deserialize, Serialize};
use rbatis::crud;
use rbatis::rbdc::datetime::DateTime;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[crud_table(table_name:"assets")]
pub struct AssetEntity {
    pub wallet_address: String,
    pub chain_id: i32,
    pub asset_type: String,
    pub symbol: String,
    pub name: String,
    pub contract_address: Option<String>,
    pub balance: Option<f64>,
    pub decimals: Option<i32>,
    pub price_usd: Option<f64>,
    pub value_usd: Option<f64>,
    pub created_at: Option<DateTime>,
    pub updated_at: Option<DateTime>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[crud_table(table_name:"nft_assets")]
pub struct NftAssetEntity {
    pub wallet_address: String,
    pub chain_id: i32,
    pub contract_address: String,
    pub token_id: String,
    pub name: String,
    pub image_url: Option<String>,
    pub metadata_url: Option<String>,
    pub created_at: Option<DateTime>,
    pub updated_at: Option<DateTime>,
}