use crate::models::user::{User, UserProfile};
use crate::utils::jwt;
use crate::utils::error::ServiceError;
use ethers::signers::Signer;
use sqlx::{Pool, Postgres};
use std::sync::Arc;

/// 用户服务，处理用户身份和资料管理
pub struct UserService {
    db: Pool<Postgres>,
}

impl UserService {
    pub fn new(db: Pool<Postgres>) -> Self {
        Self { db }
    }

    /// 使用钱包签名登录或注册
    pub async fn wallet_login(
        &self,
        wallet_address: String,
        chain_type: String,
        signature: String,
        message: String,
    ) -> Result<String, ServiceError> {
        // 验证签名
        self.verify_wallet_signature(&wallet_address, &signature, &message, &chain_type)?;

        // 检查用户是否存在，不存在则创建
        let user = self.find_or_create_user(&wallet_address, &chain_type).await?;

        // 生成JWT令牌
        let token = jwt::generate_token(user.id, &wallet_address)?;

        Ok(token)
    }

    /// 验证钱包签名
    fn verify_wallet_signature(
        &self,
        address: &str,
        signature: &str,
        message: &str,
        chain_type: &str,
    ) -> Result<bool, ServiceError> {
        match chain_type {
            "ethereum" => {
                // 使用ethers-rs验证以太坊签名
                // 实际实现会更复杂，这里简化处理
                Ok(true)
            },
            "solana" => {
                // 使用solana-client验证Solana签名
                // 实际实现会更复杂，这里简化处理
                Ok(true)
            },
            _ => Err(ServiceError::BadRequest("不支持的链类型".into())),
        }
    }

    /// 查找或创建用户
    async fn find_or_create_user(&self, wallet_address: &str, chain_type: &str) -> Result<User, ServiceError> {
        // 查询用户
        let user = sqlx::query_as!(
            User,
            r#"
            SELECT * FROM users 
            WHERE wallet_address = $1 AND chain_type = $2
            "#,
            wallet_address,
            chain_type
        )
        .fetch_optional(&self.db)
        .await?;

        // 如果用户不存在，创建新用户
        if let Some(user) = user {
            Ok(user)
        } else {
            let user = sqlx::query_as!(
                User,
                r#"
                INSERT INTO users (wallet_address, chain_type)
                VALUES ($1, $2)
                RETURNING *
                "#,
                wallet_address,
                chain_type
            )
            .fetch_one(&self.db)
            .await?;

            Ok(user)
        }
    }

    /// 更新用户资料
    pub async fn update_profile(
        &self,
        user_id: i32,
        username: Option<String>,
        nickname: Option<String>,
        avatar_cid: Option<String>,
    ) -> Result<UserProfile, ServiceError> {
        // 检查用户名是否已存在
        if let Some(username) = &username {
            let exists = sqlx::query!(
                r#"
                SELECT COUNT(*) as count FROM user_profiles 
                WHERE username = $1 AND user_id != $2
                "#,
                username,
                user_id
            )
            .fetch_one(&self.db)
            .await?
            .count
            .unwrap_or(0) > 0;

            if exists {
                return Err(ServiceError::BadRequest("用户名已存在".into()));
            }
        }

        // 更新用户资料
        let profile = sqlx::query_as!(
            UserProfile,
            r#"
            INSERT INTO user_profiles (user_id, username, nickname, avatar_cid)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT (user_id) 
            DO UPDATE SET 
                username = COALESCE($2, user_profiles.username),
                nickname = COALESCE($3, user_profiles.nickname),
                avatar_cid = COALESCE($4, user_profiles.avatar_cid),
                updated_at = NOW()
            RETURNING *
            "#,
            user_id,
            username,
            nickname,
            avatar_cid
        )
        .fetch_one(&self.db)
        .await?;

        Ok(profile)
    }

    /// 获取用户资料
    pub async fn get_profile(&self, user_id: i32) -> Result<UserProfile, ServiceError> {
        let profile = sqlx::query_as!(
            UserProfile,
            r#"
            SELECT * FROM user_profiles WHERE user_id = $1
            "#,
            user_id
        )
        .fetch_optional(&self.db)
        .await?
        .ok_or(ServiceError::NotFound("用户资料不存在".into()))?;

        Ok(profile)
    }

    /// 通过用户名获取用户资料
    pub async fn get_profile_by_username(&self, username: &str) -> Result<UserProfile, ServiceError> {
        let profile = sqlx::query_as!(
            UserProfile,
            r#"
            SELECT * FROM user_profiles WHERE username = $1
            "#,
            username
        )
        .fetch_optional(&self.db)
        .await?
        .ok_or(ServiceError::NotFound("用户资料不存在".into()))?;

        Ok(profile)
    }

    /// 通过钱包地址获取用户资料
    pub async fn get_profile_by_wallet(&self, wallet_address: &str) -> Result<UserProfile, ServiceError> {
        let profile = sqlx::query_as!(
            UserProfile,
            r#"
            SELECT p.* FROM user_profiles p
            JOIN users u ON p.user_id = u.id
            WHERE u.wallet_address = $1
            "#,
            wallet_address
        )
        .fetch_optional(&self.db)
        .await?
        .ok_or(ServiceError::NotFound("用户资料不存在".into()))?;

        Ok(profile)
    }

    /// 通过用户ID获取钱包地址
    pub async fn get_wallet_address_by_user_id(&self, user_id: i32) -> Result<String, ServiceError> {
        let user = sqlx::query!(
            r#"
            SELECT wallet_address FROM users 
            WHERE id = $1
            "#,
            user_id
        )
        .fetch_optional(&self.db)
        .await?
        .ok_or(ServiceError::NotFound("用户不存在".into()))?;

        Ok(user.wallet_address)
    }
} 