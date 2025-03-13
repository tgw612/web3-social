use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use diesel::prelude::*;
use diesel::Queryable;
use diesel::Insertable;
use uuid::Uuid;
use crate::schema::posts; // 假设你的表名是 posts

#[derive(Debug, Serialize, Deserialize, Queryable, Insertable)]
#[diesel(table_name = posts)]
pub struct Post {
    pub id: Uuid,
    pub user_id: Uuid,
    pub content: String,
    pub images_ipfs_cids: Option<Vec<String>>, // IPFS内容IDs
    pub arweave_tx_id: Option<String>,         // Arweave交易ID
    pub transaction_hash: Option<String>,      // 区块链交易哈希
    pub transaction_chain: Option<String>,     // 交易所在链
    pub like_count: i32,
    pub comment_count: i32,
    pub tags: Option<Vec<String>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreatePostRequest {
    pub content: String,
    pub images_ipfs_cids: Option<Vec<String>>,
    pub transaction_hash: Option<String>,
    pub transaction_chain: Option<String>,
    pub tags: Option<Vec<String>>,
    pub signature: String, // 钱包签名，用于验证身份
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PostResponse {
    pub id: String,
    pub user_id: String,
    pub username: String,
    pub nickname: Option<String>,
    pub avatar_ipfs_url: Option<String>,
    pub wallet_address: String,
    pub content: String,
    pub images_urls: Option<Vec<String>>,
    pub arweave_url: Option<String>,
    pub transaction_hash: Option<String>,
    pub transaction_chain: Option<String>,
    pub transaction_url: Option<String>,
    pub like_count: i32,
    pub comment_count: i32,
    pub tags: Option<Vec<String>>,
    pub created_at: DateTime<Utc>,
    pub user_liked: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PostsQueryParams {
    pub page: Option<i64>,
    pub per_page: Option<i64>,
    pub user_id: Option<Uuid>,
    pub tag: Option<String>,
    pub sort_by: Option<String>, // "hot" or "new"
}

impl Post {
    pub fn to_response(
        &self,
        ipfs_gateway_url: &str,
        arweave_node_url: &str,
        username: String,
        nickname: Option<String>,
        avatar_ipfs_cid: Option<String>,
        wallet_address: String,
        user_liked: bool,
    ) -> PostResponse {
        let images_urls = self.images_ipfs_cids.as_ref().map(|cids| {
            cids.iter()
                .map(|cid| format!("{}/ipfs/{}", ipfs_gateway_url, cid))
                .collect()
        });

        let avatar_ipfs_url = avatar_ipfs_cid.map(|cid| {
            format!("{}/ipfs/{}", ipfs_gateway_url, cid)
        });

        let arweave_url = self.arweave_tx_id.as_ref().map(|tx_id| {
            format!("{}/{}", arweave_node_url, tx_id)
        });

        let transaction_url = match (&self.transaction_hash, &self.transaction_chain) {
            (Some(hash), Some(chain)) if chain == "ETH" => {
                Some(format!("https://etherscan.io/tx/{}", hash))
            }
            (Some(hash), Some(chain)) if chain == "SOL" => {
                Some(format!("https://explorer.solana.com/tx/{}", hash))
            }
            _ => None,
        };

        PostResponse {
            id: self.id.to_string(),
            user_id: self.user_id.to_string(),
            username,
            nickname,
            avatar_ipfs_url,
            wallet_address,
            content: self.content.clone(),
            images_urls,
            arweave_url,
            transaction_hash: self.transaction_hash.clone(),
            transaction_chain: self.transaction_chain.clone(),
            transaction_url,
            like_count: self.like_count,
            comment_count: self.comment_count,
            tags: self.tags.clone(),
            created_at: self.created_at,
            user_liked,
        }
    }
} 