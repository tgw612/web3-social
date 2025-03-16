use crate::models::user::{User, UserProfile};
use crate::utils::jwt;
use crate::utils::error::ServiceError;
use diesel::pg::PgConnection;
use std::sync::Arc;
use std::sync::Mutex;
// user_service.rs 或其他使用查询的模块
use diesel::{BoolExpressionMethods, ExpressionMethods};  // 关键导入
use diesel::QueryDsl;  

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
    async fn find_or_create_user(&self, wallet_address: &str, wallet_chain: &str) -> Result<User, ServiceError> {
        use crate::schema::users::dsl::*;
             // 如果用到其他查询方法也需导入
        let mut conn = self.db.lock().unwrap();
        
        let user = users
            .filter(wallet_address.eq(wallet_address))
            .first::<User>(&mut *conn)
            .optional()?;

        if let Some(user) = user {
            Ok(user)
        } else {
            diesel::insert_into(users)
                .values(&User {
                    wallet_address:wallet_address.to_string(),
                    wallet_chain: wallet_chain.to_string(),
                    id: uuid::Uuid::new_v4(),
                    username: "".to_string(),
                    nickname: "".to_string(),
                    avatar_ipfs_cid: None,
                    created_at: chrono::NaiveDateTime::now(),
                    updated_at: chrono::NaiveDateTime::now(),
                })
                .get_result(&mut *conn)
                .map_err(|_| ServiceError::InternalServerError)
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
        use crate::schema::user_profiles::dsl::{user_profiles, user_id as prof_user_id, username as prof_username};
        let mut conn = self.db.lock().unwrap();

        if let Some(username_val) = &username {
            let exists = user_profiles
                .filter(prof_username.eq(username_val).and(prof_user_id.ne(user_id)))
                .count()
                .get_result::<i64>(&mut *conn)? > 0;

            if exists {
                return Err(ServiceError::BadRequest("用户名已存在".into()));
            }
        }

        diesel::insert_into(user_profiles)
            .values((
                prof_user_id.eq(user_id),
                prof_username.eq(&username),
                nickname.eq(&nickname),
                avatar_cid.eq(&avatar_cid),
            ))
            .on_conflict(prof_user_id)
            .do_update()
            .set((
                prof_username.eq(&username),
                nickname.eq(&nickname),
                avatar_cid.eq(&avatar_cid),
                updated_at.eq(diesel::dsl::now),
            ))
            .get_result(&mut *conn)
            .map_err(|_| ServiceError::InternalServerError)
    }

    /// 获取用户资料
    pub async fn get_profile(&self, user_id: i32) -> Result<UserProfile, ServiceError> {
        use crate::schema::user_profiles::dsl::*;
        let mut conn = self.db.lock().unwrap();
        
        user_profiles
            .filter(user_id.eq(user_id))
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
            .first::<UserProfile>(&mut *conn)
            .optional()?
            .ok_or(ServiceError::NotFound("用户资料不存在".into()))
    }

    /// 通过钱包地址获取用户资料
    pub async fn get_profile_by_wallet(&self, wallet_address: &str) -> Result<UserProfile, ServiceError> {
        use crate::schema::users::dsl::{users, wallet_address as user_wallet};
        use crate::schema::user_profiles::dsl::*;
        let mut conn = self.db.lock().unwrap();
        
        user_profiles
            .inner_join(users.on(prof_user_id.eq(user_id_col)))
            .filter(user_wallet.eq(wallet_address))
            .select(user_profiles::all_columns())
            .first::<UserProfile>(&mut *conn)
            .optional()?
            .ok_or(ServiceError::NotFound("用户资料不存在".into()))
    }

    /// 通过用户ID获取钱包地址
    pub async fn get_wallet_address_by_user_id(&self, user_id: i32) -> Result<String, ServiceError> {
        use crate::schema::users::dsl::{users, id, wallet_address};
        let mut conn = self.db.lock().unwrap();
        
        users
            .filter(user_id_col.eq(user_id))
            .select(user_wallet)
            .first::<String>(&mut *conn)
            .optional()?
            .ok_or(ServiceError::NotFound("用户不存在".into()))
    }
} 