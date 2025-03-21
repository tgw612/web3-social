use crate::services::user_service::UserService;
use crate::utils::error::ServiceError;
use actix_web::{web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Deserialize)]
pub struct WalletLoginRequest {
    wallet_address: String,
    chain_type: String,
    signature: String,
    message: String,
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    token: String,
    user_id: Option<i32>,
}

/// 钱包登录接口
pub async fn wallet_login(
    data: web::Json<WalletLoginRequest>,
    user_service: web::Data<Arc<UserService>>,
) -> impl Responder {
    // 调用用户服务进行钱包登录
    match user_service.wallet_login(
        data.wallet_address.clone(),
        data.chain_type.clone(),
        data.signature.clone(),
        data.message.clone(),
    ).await {
        Ok(token) => {
            // 登录成功，返回JWT令牌
            HttpResponse::Ok().json(LoginResponse {
                token,
                user_id: None, // 实际应用中可能需要从JWT中解析user_id
            })
        },
        Err(err) => {
            // 登录失败，返回错误信息
            match err {
                ServiceError::AuthenticationError(_) => {
                    HttpResponse::Unauthorized().json(serde_json::json!({
                        "status": "error",
                        "message": "签名验证失败"
                    }))
                },
                _ => {
                    HttpResponse::InternalServerError().json(serde_json::json!({
                        "status": "error",
                        "message": format!("登录失败: {}", err)
                    }))
                }
            }
        }
    }
}

/// 检查令牌是否有效（用于客户端验证会话状态）
pub async fn verify_token() -> impl Responder {
    // 该接口会通过中间件验证令牌，如果能到达这里说明令牌有效
    // 不需要额外逻辑
    HttpResponse::Ok().json(serde_json::json!({
        "status": "success",
        "message": "令牌有效"
    }))
}

/// 配置Auth路由
pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/auth")
            .route("/wallet-login", web::post().to(wallet_login))
            .route("/verify", web::get().to(verify_token))
    );
} 