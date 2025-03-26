use crate::models::rbatis_entities::{UserEntity, UserProfileEntity};
use crate::utils::error::ServiceError;
use crate::utils::jwt;
use chrono::{DateTime as ChronoDateTime, Utc};
use rbatis::rbdc::datetime::DateTime;
use rbatis::RBatis;
use std::sync::Arc;
use uuid::Uuid;

/// 用户服务，处理用户身份和资料管理
pub struct UserService {
    db: Arc<RBatis>,
}

impl UserService {
    pub fn new(db: Arc<RBatis>) -> Self {
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
        let user: UserEntity = self
            .find_or_create_user(&wallet_address, &chain_type)
            .await?;
        // 生成JWT令牌
        let token: String = jwt::generate_token(user.id, &wallet_address)?;

        Ok(token)
    }

    /// 验证钱包签名
    fn verify_wallet_signature(
        &self,
        address: &str,
        signature: &str,
        message: &str,
        wallet_chain: &str,
    ) -> Result<bool, ServiceError> {
        match wallet_chain {
            "ethereum" => {
                // 使用ethers-rs验证以太坊签名
                // 实际实现会更复杂，这里简化处理
                Ok(true)
            }
            "solana" => {
                // 使用solana-client验证Solana签名
                // 实际实现会更复杂，这里简化处理
                Ok(true)
            }
            _ => Err(ServiceError::BadRequest("不支持的链类型".into())),
        }
    }

    /// 查找或创建用户
    async fn find_or_create_user(
        &self,
        wallet_address_val: &String,
        wallet_chain_val: &str,
    ) -> Result<UserEntity, ServiceError> {
        // 使用self.db而不是创建新的RBatis实例
        let rb: RBatis = RBatis::new();
        let user_entity = UserEntity::select_by_column(&rb, "wallet_address", wallet_address_val)
            .await
            .map_err(|_| ServiceError::InternalServerError)?
            .first()
            .cloned();

        if let Some(entity) = user_entity {
            Ok(entity)
        } else {
            let new_user_entity = UserEntity {
                id: Uuid::new_v4(),
                username: "".to_string(),
                nickname: Some("".to_string()),
                wallet_address: wallet_address_val.clone(),
                wallet_chain: wallet_chain_val.to_string(),
                avatar_ipfs_cid: Some("".to_string()),
                created_at: DateTime::now(),
                updated_at: DateTime::now(),
            };

            UserEntity::insert(&rb, &new_user_entity)
                .await
                .map_err(|_| ServiceError::InternalServerError)?;

            Ok(new_user_entity)
        }
    }

    /// 更新用户资料
    pub async fn update_profile(
        &self,
        user_id: String,
        username: Option<String>,
        nickname: Option<String>,
        avatar_cid: Option<String>,
    ) -> Result<String, ServiceError> {
        if let Some(username_val) = &username {
            let rb: RBatis = RBatis::new();
            let profile: Vec<UserProfileEntity> =
                UserProfileEntity::select_by_column(&rb, "username", &username_val)
                    .await
                    .map_err(|_| ServiceError::InternalServerError)?;
            let profile_entity = profile.first().cloned().unwrap();
            if profile.first().is_none() {
                return Err(ServiceError::BadRequest("用户名已存在".into()));
            }
            UserProfileEntity::update_by_column(&rb, &profile_entity, "nickname")
                .await
                .map_err(|_| ServiceError::InternalServerError)?;

            Ok(String::from("修改成功！"))
        } else {
            // 处理没有提供用户名的情况
            Err(ServiceError::BadRequest("用户名不能为空".into()))
        }
    }

    /// 获取用户资料
    pub async fn get_profile(
        &self,
        user_id_val: String,
    ) -> Result<UserProfileEntity, ServiceError> {
        // 使用rbatis查询用户资料
        let rb: RBatis = RBatis::new();
        let profile: Vec<UserProfileEntity> =
            UserProfileEntity::select_by_column(&rb, "user_id", &user_id_val)
                .await
                .map_err(|_| ServiceError::InternalServerError)?;
        Ok(profile.first().cloned().unwrap())
    }

    /// 通过用户名获取用户资料
    pub async fn get_profile_by_username(
        &self,
        username_val: &str,
    ) -> Result<UserProfileEntity, ServiceError> {
        // 使用rbatis查询用户资料
        let rb: RBatis = RBatis::new();
        let profile = UserProfileEntity::select_by_column(&rb, "username", username_val)
            .await
            .map_err(|_| ServiceError::InternalServerError)?;

        Ok(profile.first().cloned().unwrap())
    }

    /// 通过钱包地址获取用户资料
    pub async fn get_profile_by_wallet(
        &self,
        wallet_address_val: &str,
    ) -> Result<UserProfileEntity, ServiceError> {
        // 使用rbatis查询用户资料
        let rb: RBatis = RBatis::new();
        let profile: Vec<UserProfileEntity> =
            UserProfileEntity::select_by_column(&rb, "wallet_address", wallet_address_val)
                .await
                .map_err(|_| ServiceError::InternalServerError)?;

        Ok(profile.first().cloned().unwrap())
    }

    /// 通过用户ID获取钱包地址
    pub async fn get_wallet_address_by_user_id(
        &self,
        user_id_val: String,
    ) -> Result<String, ServiceError> {
        // 解析UUID
        let user_id_uuid = uuid::Uuid::parse_str(&user_id_val)
            .map_err(|_| ServiceError::BadRequest("无效的用户ID".into()))?;
        let rb: RBatis = RBatis::new();
        // 使用rbatis查询用户

        let user_entity: Vec<UserEntity> = UserEntity::select_by_column(&rb, "id", user_id_uuid)
            .await
            .map_err(|_| ServiceError::InternalServerError)?;

        Ok(user_entity.first().unwrap().wallet_address.clone())
    }
}
