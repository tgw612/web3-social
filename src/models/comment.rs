use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use diesel::prelude::*;
use diesel::Queryable;
use diesel::Insertable;
use crate::schema::comments; // 假设你的表名是 comments

#[derive(Debug, Serialize, Deserialize, Queryable, Insertable)]
#[diesel(table_name = comments)]
pub struct Comment {
    pub id: Uuid,
    pub post_id: Uuid,
    pub user_id: Uuid,
    pub parent_id: Option<Uuid>, // 回复其他评论时，存储父评论ID
    pub content: String,
    pub arweave_tx_id: Option<String>, // Arweave交易ID
    pub like_count: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateCommentRequest {
    pub post_id: String,
    pub parent_id: Option<String>,
    pub content: String,
    pub signature: String, // 钱包签名，用于验证身份
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CommentResponse {
    pub id: String,
    pub post_id: String,
    pub user_id: String,
    pub parent_id: Option<String>,
    pub username: String,
    pub nickname: Option<String>,
    pub avatar_ipfs_url: Option<String>,
    pub wallet_address: String,
    pub content: String,
    pub arweave_url: Option<String>,
    pub like_count: i32,
    pub created_at: DateTime<Utc>,
    pub user_liked: bool,
    pub replies: Option<Vec<Box<CommentResponse>>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CommentsQueryParams {
    pub post_id: String,
    pub page: Option<i64>,
    pub per_page: Option<i64>,
    pub sort_by: Option<String>, // "hot" or "new"
}

impl Comment {
    pub fn to_response(
        &self,
        arweave_node_url: &str,
        ipfs_gateway_url: &str,
        username: String,
        nickname: Option<String>,
        avatar_ipfs_cid: Option<String>,
        wallet_address: String,
        user_liked: bool,
        replies: Option<Vec<Box<CommentResponse>>>,
    ) -> CommentResponse {
        let avatar_ipfs_url = avatar_ipfs_cid.map(|cid| {
            format!("{}/ipfs/{}", ipfs_gateway_url, cid)
        });

        let arweave_url = self.arweave_tx_id.as_ref().map(|tx_id| {
            format!("{}/{}", arweave_node_url, tx_id)
        });

        CommentResponse {
            id: self.id.to_string(),
            post_id: self.post_id.to_string(),
            user_id: self.user_id.to_string(),
            parent_id: self.parent_id.map(|id| id.to_string()),
            username,
            nickname,
            avatar_ipfs_url,
            wallet_address,
            content: self.content.clone(),
            arweave_url,
            like_count: self.like_count,
            created_at: self.created_at,
            user_liked,
            replies,
        }
    }
} 