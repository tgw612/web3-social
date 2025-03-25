use crate::{models::user::{User, UserProfile}, utils::error::ServiceError};
use crate::utils::jwt;
use crate::models::rbatis_entities::{UserEntity, UserProfileEntity};
use std::sync::Arc;
use rbatis::RBatis;
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
        let user: User = self.find_or_create_user(&wallet_address, &chain_type).await?;

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
    async fn find_or_create_user(&self, wallet_address_val: &String, wallet_chain_val: &str) -> Result<User, ServiceError> {
        // 使用rbatis查询用户
        let user_entity = self.db.query_by_column::<UserEntity>("wallet_address", wallet_address_val)
            .await
            .map_err(|e| ServiceError::InternalServerError)?;
        
        if let Some(entity) = user_entity {
            // 将实体转换为User模型
            let user = User {
                id: entity.id,
                username: entity.username,
                nickname: entity.nickname,
                wallet_address: entity.wallet_address,
                wallet_chain: entity.wallet_chain,
                avatar_ipfs_cid: entity.avatar_ipfs_cid,
                created_at: entity.created_at.into(),
                updated_at: entity.updated_at.into(),
            };
            Ok(user)
        } else {
            use chrono::{DateTime, Utc};
            // 创建新用户实体
            let new_user_entity = UserEntity {
                id: uuid::Uuid::new_v4(),
                username: "".to_string(),
                nickname: Some("".to_string()),
                wallet_address: wallet_address_val.clone(),
                wallet_chain: wallet_chain_val.to_string(),
                avatar_ipfs_cid: Some("".to_string()),
                created_at: rbatis::rbdc::datetime::DateTime::now(),
                updated_at: rbatis::rbdc::datetime::DateTime::now(),
            };
            
            // 插入新用户
            self.db.save(&new_user_entity, &[])
                .await
                .map_err(|_| ServiceError::InternalServerError)?;
            
            // 将实体转换为User模型
            let user = User {
                id: new_user_entity.id,
                username: new_user_entity.username,
                nickname: new_user_entity.nickname,
                wallet_address: new_user_entity.wallet_address,
                wallet_chain: new_user_entity.wallet_chain,
                avatar_ipfs_cid: new_user_entity.avatar_ipfs_cid,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            };
            
            Ok(user)
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
            // 使用rbatis查询用户名是否已存在
            let wrapper = self.db.new_wrapper()
                .eq("username", username_val);
            
            let count = self.db.query_count_by_wrapper::<UserProfileEntity>(wrapper)
                .await
                .map_err(|_| ServiceError::InternalServerError)?;

            if count > 0 {
                return Err(ServiceError::BadRequest("用户名已存在".into()));
            } else {
                // 更新用户资料
                let wrapper = self.db.new_wrapper()
                    .eq("user_id", &user_id);
                
                let mut update_values = rbatis::py_sql::PyDict::new();
                update_values.insert("username", username_val);
                if let Some(nick) = &nickname {
                    update_values.insert("nickname", nick);
                }
                if let Some(avatar) = &avatar_cid {
                    update_values.insert("avatar_cid", avatar);
                }
                update_values.insert("updated_at", rbatis::rbdc::datetime::DateTime::now());
                
                self.db.update_by_wrapper::<UserProfileEntity>(&update_values, wrapper)
                    .await
                    .map_err(|_| ServiceError::InternalServerError)?;
                
                return Ok(String::from("修改成功！"))
            }
        } else {
            // 处理没有提供用户名的情况
            return Err(ServiceError::BadRequest("用户名不能为空".into()));
        }
    }   

    /// 获取用户资料
    pub async fn get_profile(&self, user_id_val: String) -> Result<UserProfile, ServiceError> {
        // 使用rbatis查询用户资料
        let wrapper = self.db.new_wrapper()
            .eq("user_id", &user_id_val);
        
        let profile = self.db.query_by_wrapper::<UserProfileEntity>(wrapper)
            .await
            .map_err(|_| ServiceError::InternalServerError)?
            .ok_or(ServiceError::NotFound("用户资料不存在".into()))?;
        
        // 将实体转换为UserProfile模型
        let user_profile = UserProfile {
            id: profile.id,
            user_id: profile.user_id,
            username: profile.username,
            nickname: profile.nickname,
            wallet_address: profile.wallet_address,
            avatar_cid: profile.avatar_cid,
            created_at: profile.created_at.into(),
            updated_at: profile.updated_at.into(),
        };
        
        Ok(user_profile)
    }

    /// 通过用户名获取用户资料
    pub async fn get_profile_by_username(&self, username_val: &str) -> Result<UserProfile, ServiceError> {
        // 使用rbatis查询用户资料
        let wrapper = self.db.new_wrapper()
            .eq("username", username_val);
        
        let profile = self.db.query_by_wrapper::<UserProfileEntity>(wrapper)
            .await
            .map_err(|_| ServiceError::InternalServerError)?
            .ok_or(ServiceError::NotFound("用户资料不存在".into()))?;
        
        // 将实体转换为UserProfile模型
        let user_profile = UserProfile {
            id: profile.id,
            user_id: profile.user_id,
            username: profile.username,
            nickname: profile.nickname,
            wallet_address: profile.wallet_address,
            avatar_cid: profile.avatar_cid,
            created_at: profile.created_at.into(),
            updated_at: profile.updated_at.into(),
        };
        
        Ok(user_profile)
    }

    /// 通过钱包地址获取用户资料
    pub async fn get_profile_by_wallet(&self, wallet_address_val: &str) -> Result<UserProfile, ServiceError> {
        // 使用rbatis查询用户资料
        let wrapper = self.db.new_wrapper()
            .eq("wallet_address", wallet_address_val);
        
        let profile = self.db.query_by_wrapper::<UserProfileEntity>(wrapper)
            .await
            .map_err(|_| ServiceError::InternalServerError)?
            .ok_or(ServiceError::NotFound("用户资料不存在".into()))?;
        
        // 将实体转换为UserProfile模型
        let user_profile = UserProfile {
            id: profile.id,
            user_id: profile.user_id,
            username: profile.username,
            nickname: profile.nickname,
            wallet_address: profile.wallet_address,
            avatar_cid: profile.avatar_cid,
            created_at: profile.created_at.into(),
            updated_at: profile.updated_at.into(),
        };
        
        Ok(user_profile)
    }
                                                                                                                                                                  
    /// 通过用户ID获取钱包地址
    pub async fn get_wallet_address_by_user_id(&self, user_id_val: String) -> Result<String, ServiceError> {
        // 解析UUID
        let user_id_uuid = uuid::Uuid::parse_str(&user_id_val)
            .map_err(|_| ServiceError::BadRequest("无效的用户ID".into()))?;
        
        // 使用rbatis查询用户
        let wrapper = self.db.new_wrapper()
            .eq("id", user_id_uuid);
        
        let user_entity = self.db.fetch_by_wrapper::<UserEntity>(wrapper)
            .await
            .map_err(|_| ServiceError::InternalServerError)?
            .ok_or(ServiceError::NotFound("用户不存在".into()))?;
        
        Ok(user.wallet_address)
    }
}