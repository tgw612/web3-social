use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// 用于生成登录挑战的请求
#[derive(Debug, Serialize, Deserialize)]
pub struct ChallengeRequest {
    pub wallet_address: String,
    pub wallet_chain: String,
}

// 登录挑战信息
#[derive(Debug, Serialize, Deserialize)]
pub struct Challenge {
    pub id: Uuid,
    pub wallet_address: String,
    pub wallet_chain: String,
    pub nonce: String,  // 随机生成的挑战码
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

// 验证登录签名的请求
#[derive(Debug, Serialize, Deserialize)]
pub struct VerifySignatureRequest {
    pub wallet_address: String,
    pub wallet_chain: String,
    pub signature: String,  // 钱包对挑战码的签名
    pub challenge_id: String,
}

// JWT令牌内容
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,  // 用户ID
    pub wallet_address: String,
    pub wallet_chain: String,
    pub exp: usize,   // 过期时间戳
    pub iat: usize,   // 颁发时间戳
}

// 登录成功返回
#[derive(Debug, Serialize, Deserialize)]
pub struct LoginResponse {
    pub token: String,
    pub user_id: String,
    pub username: Option<String>,
    pub wallet_address: String,
    pub wallet_chain: String,
    pub is_new_user: bool,
}

impl Challenge {
    pub fn new(wallet_address: String, wallet_chain: String, nonce: String) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            wallet_address,
            wallet_chain,
            nonce,
            created_at: now,
            expires_at: now + Duration::minutes(15), // 15分钟有效期
        }
    }

    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }
} 