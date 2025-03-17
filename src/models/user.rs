use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use diesel::Queryable;
use diesel::Insertable;
use uuid::Uuid;
use crate::schema::users; // 假设你的表名是 users
use diesel::prelude::*;
use diesel::Selectable;

#[derive(Debug, Serialize, Deserialize, Queryable, Insertable, Selectable)]
#[diesel(table_name = users)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub nickname: Option<String>,
    pub wallet_address: String,
    pub wallet_chain: String, // ETH, SOL, etc.
    pub avatar_ipfs_cid: Option<String>, // IPFS内容ID
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserAssetSummary {
    pub total_value_usd: f64,
    pub assets_distribution: Vec<AssetDistribution>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AssetDistribution {
    pub symbol: String,
    pub name: String,
    pub value_usd: f64,
    pub percentage: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateUserRequest {
    pub username: String,
    pub nickname: Option<String>,
    pub wallet_address: String,
    pub wallet_chain: String,
    pub avatar_ipfs_cid: Option<String>,
    pub signature: String, // 钱包签名，用于验证地址所有权
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateUserRequest {
    pub nickname: Option<String>,
    pub avatar_ipfs_cid: Option<String>,
    pub signature: String, // 钱包签名，用于验证地址所有权
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserResponse {
    pub id: String,
    pub username: String,
    pub nickname: Option<String>,
    pub wallet_address: String,
    pub wallet_chain: String,
    pub avatar_ipfs_url: Option<String>, // 完整的IPFS网关URL
    pub created_at: DateTime<Utc>,
    pub assets: Option<UserAssetSummary>,
}

impl User {
    pub fn to_response(&self, ipfs_gateway_url: &str, assets: Option<UserAssetSummary>) -> UserResponse {
        let avatar_ipfs_url = self.avatar_ipfs_cid.as_ref().map(|cid| {
            format!("{}/ipfs/{}", ipfs_gateway_url, cid)
        });

        UserResponse {
            id: self.id.to_string(),
            username: self.username.clone(),
            nickname: self.nickname.clone(),
            wallet_address: self.wallet_address.clone(),
            wallet_chain: self.wallet_chain.clone(),
            avatar_ipfs_url,
            created_at: self.created_at,
            assets,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Queryable, Selectable)]
#[diesel(table_name = crate::schema::user_profiles)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct UserProfile {
    pub id: i32,
    pub user_id: i32,
    pub username: Option<String>,
    pub nickname: Option<String>,
    pub avatar_cid: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
