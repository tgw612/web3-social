use serde::{Deserialize, Serialize};
use rbatis::rbdc::datetime::DateTime;
use uuid::Uuid;
use rbatis::crud;

// 为User结构体自动生成CRUD方法
// 如果指定了表名，则使用指定的表名；否则，使用结构体名称的蛇形命名法作为表名
crud!(AssetEntity {}); // 表名为user
#[derive(Debug, Clone, Serialize, Deserialize)]
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
crud!(NftAssetEntity {});
#[derive(Debug, Clone, Serialize, Deserialize)]
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
crud!(UserEntity {}); 
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserEntity {
    pub id: Uuid,
    pub username: String,
    pub nickname: Option<String>,
    pub wallet_address: String,
    pub wallet_chain: String,
    pub avatar_ipfs_cid: Option<String>,
    pub created_at: DateTime,
    pub updated_at: DateTime,
}
crud!(UserProfileEntity {}); 
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProfileEntity {
    pub id: i32,
    pub user_id: Uuid,
    pub username: Option<String>,
    pub nickname: Option<String>,
    pub wallet_address: String,
    pub avatar_cid: Option<String>,
    pub created_at: DateTime,
    pub updated_at: DateTime,
}
crud!(PostEntity {});
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostEntity {
    pub id: i32,
    pub user_id: String,
    pub content: String,
    pub images_ipfs_cids: Option<Vec<String>>,
    pub arweave_tx_id: Option<String>,
    pub transaction_hash: Option<String>,
    pub transaction_chain: Option<String>,
    pub like_count: i32,
    pub comment_count: i32,
    pub tags: Option<Vec<String>>,
    pub created_at: DateTime,
    pub updated_at: DateTime,
}

crud!(CommentEntity {});
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommentEntity {
    pub id: Uuid,
    pub post_id: String,
    pub user_id: String,
    pub parent_id: Option<String>,
    pub content: String,
    pub arweave_tx_id: Option<String>,
    pub like_count: i32,
    pub created_at: DateTime,
    pub updated_at: DateTime,
}

crud!(UserLikeEntity {});
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserLikeEntity {
    pub id: Uuid,
    pub user_id: Uuid,
    pub post_id: Option<Uuid>,
    pub comment_id: Option<Uuid>,
    pub created_at: DateTime,
}

crud!(AuthChallengeEntity {});
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthChallengeEntity {
    pub id: Uuid,
    pub wallet_address: String,
    pub wallet_chain: String,
    pub nonce: String,
    pub created_at: DateTime,
    pub expires_at: DateTime,
}

crud!(TagEntity {});
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TagEntity {
    pub id: i32,
    pub name: String,
}