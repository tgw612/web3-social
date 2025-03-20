use crate::{models::user::{User, UserProfile}, utils::error::ServiceError};
use crate::utils::jwt;
use diesel::associations::HasTable;
use diesel::pg::PgConnection;
use std::sync::Arc;
use std::sync::Mutex;
// user_service.rs 或其他使用查询的模块
use diesel::{BoolExpressionMethods, ExpressionMethods};  // 关键导入
use diesel::QueryDsl;  
use diesel::RunQueryDsl; // 关键trait
use diesel::OptionalExtension; 


/// 用户服务，处理用户身份和资料管理
pub struct UserService {
    db: Arc<Mutex<PgConnection>>,
}

impl UserService {
    pub fn new(db: Arc<Mutex<PgConnection>>) -> Self {
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
        use crate::schema::users::dsl::*;
             // 如果用到其他查询方法也需导入
        let mut conn = self.db.lock().unwrap();
        
        let user: Option<User> = users
            .filter(wallet_address.eq(wallet_address))  
            .filter(wallet_chain.eq(wallet_chain))
            .first::<User>(&mut *conn)
            .optional()?;

        if let Some(user) = user {
            Ok(user)
        } else {
            use chrono::{DateTime, Utc};
            let new_user: User =User{
                wallet_address: wallet_address_val.clone(),
                wallet_chain: wallet_chain_val.to_string(),
                id: uuid::Uuid::new_v4(),
                username: "".to_string(),
                nickname: Some("".to_string()),
                avatar_ipfs_cid: Some("".to_string()),
                created_at: Utc::now(),
                updated_at: Utc::now()
            };
            diesel::insert_into(users::table())
                .values(&new_user)
                .get_result(&mut *conn)
                .map_err(|_| ServiceError::InternalServerError)
        }
    }

    /// 更新用户资料
    pub async fn update_profile(    
        &self,
        user_id: &String,
        username: Option<String>,
        nickname: Option<String>,
        avatar_cid: Option<String>,
    ) -> Result<String, ServiceError> {
        use crate::schema::user_profiles::dsl::{user_profiles, user_id as prof_user_id, username as prof_username};
        let mut conn = self.db.lock().unwrap();
        if let Some(username_val) = &username {
            let exists: bool = user_profiles
                .filter(prof_username.eq(username_val).and(prof_user_id.ne(prof_user_id)))
                .count()
                .get_result::<i64>(&mut *conn)? > 0;

            if exists {
                return Err(ServiceError::BadRequest("用户名已存在".into()));
            } else {
                return Ok(String::from("修改成功！"))
            }
        } else {
            // 处理没有提供用户名的情况
            return Err(ServiceError::BadRequest("用户名不能为空".into()));
        }
    }   

    /// 获取用户资料
    pub async fn get_profile(&self, user_id: i32) -> Result<UserProfile, ServiceError> {
        use crate::schema::user_profiles::dsl::*;
        let mut conn = self.db.lock().unwrap();
        
        user_profiles
            .filter(user_id.eq(user_id))
            .select((id, user_id, username, nickname, avatar_cid, created_at, updated_at))
            .first::<UserProfile>(&mut *conn)
            .optional()?
            .ok_or(ServiceError::NotFound("用户资料不存在".into()))
    }

    /// 通过用户名获取用户资料
    pub async fn get_profile_by_username(&self, username: &str) -> Result<UserProfile, ServiceError> {
        use crate::schema::user_profiles::dsl::*;
        let mut conn = self.db.lock().unwrap();
        
        user_profiles
            .filter(username.eq(username))
            .select((id, user_id, username, nickname, avatar_cid, created_at, updated_at))  
            .first::<UserProfile>(&mut *conn)
            .optional()?
            .ok_or(ServiceError::NotFound("用户资料不存在".into()))
    }

    /// 通过钱包地址获取用户资料
    pub async fn get_profile_by_wallet(&self, wallet_address: &str) -> Result<UserProfile, ServiceError> {
        use crate::schema::user_profiles::dsl::*;
        let mut conn = self.db.lock().unwrap();
        
        user_profiles
            .filter(wallet_address.eq(wallet_address))
            .select((id, user_id, username, nickname, avatar_cid, created_at, updated_at)) // 明确选择字段
            .first::<UserProfile>(&mut *conn)
            .optional()?
            .ok_or(ServiceError::NotFound("用户资料不存在".into()))
    }
                                                                                                                                                                  
    /// 通过用户ID获取钱包地址
    pub async fn get_wallet_address_by_user_id(&self, user_id: &uuid::Uuid) -> Result<String, ServiceError> {
        use crate::schema::users::dsl::{users, id, wallet_address};
        let mut conn = self.db.lock().unwrap();
        
        let user_wallet_address: String = users
            .filter(id.eq(user_id))
            .select(wallet_address)
            .first::<String>(&mut *conn)
            .optional()?
            .ok_or(ServiceError::NotFound("用户不存在".into()))?;

        Ok(user_wallet_address)
    }
} 