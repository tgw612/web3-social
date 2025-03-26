use crate::middlewares::auth::AuthenticatedUser;
use crate::utils::error::ServiceError;
use crate::services::user_service::UserService;
use actix_web::{web, HttpResponse, Responder};
use serde::{Deserialize};
use std::sync::Arc;

#[derive(Debug, Deserialize)]
pub struct UpdateProfileRequest {
    username: Option<String>,
    nickname: Option<String>,
    avatar_data: Option<String>, // Base64编码的图像数据
}

/// 获取当前用户资料
pub async fn get_current_profile(
    auth_user: AuthenticatedUser,
    user_service: web::Data<Arc<UserService>>,

) -> impl Responder {
    match user_service.get_profile(auth_user.user_id).await {
        Ok(profile) => HttpResponse::Ok().json(profile),
        Err(err) => match err {
            ServiceError::NotFound(_) => HttpResponse::NotFound().json(serde_json::json!({
                "status": "error",
                "message": "用户资料不存在"
            })),
            _ => HttpResponse::InternalServerError().json(serde_json::json!({
                "status": "error",
                "message": format!("获取用户资料失败: {}", err)
            })),
        },
    }
}

/// 更新当前用户资料
pub async fn update_profile(
    auth_user: AuthenticatedUser,
    data: web::Json<UpdateProfileRequest>,
    user_service: web::Data<Arc<UserService>>,
) -> impl Responder {
    // 调用用户服务更新资料
    match user_service
        .update_profile(
            auth_user.user_id,
            data.username.clone(),
            data.nickname.clone(),
            None, // 头像CID暂时为空，实际应用中需要先上传到IPFS
        )
        .await
    {
        Ok(profile) => HttpResponse::Ok().json(profile),
        Err(err) => match err {
            ServiceError::BadRequest(msg) => HttpResponse::BadRequest().json(serde_json::json!({
                "status": "error",
                "message": msg
            })),
            _ => HttpResponse::InternalServerError().json(serde_json::json!({
                "status": "error",
                "message": format!("更新用户资料失败: {}", err)
            })),
        },
    }
}

/// 通过钱包地址获取用户资料
// pub async fn get_profile_by_wallet(
//     path: web::Path<String>,
//     user_service: web::Data<Arc<UserService>>,
//     asset_service: web::Data<Arc<AssetService>>,
// ) -> impl Responder {
//     let wallet_address = path.into_inner();

//     // 获取用户资料
//     let profile = match user_service.get_profile_by_wallet(&wallet_address).await {
//         Ok(profile) => profile,
//         Err(err) => {
//             return match err {
//                 ServiceError::NotFound(_) => HttpResponse::NotFound().json(serde_json::json!({
//                     "status": "error",
//                     "message": "用户不存在"
//                 })),
//                 _ => HttpResponse::InternalServerError().json(serde_json::json!({
//                     "status": "error",
//                     "message": format!("获取用户资料失败: {}", err)
//                 })),
//             }
//         }
//     };

//     // 获取用户资产总价值
//     let total_value = match asset_service.get_total_value(&wallet_address).await {
//         Ok(value) => value,
//         Err(_) => 0.0, // 如果获取失败，默认为0
//     };

//     // 构建响应
//     HttpResponse::Ok().json(serde_json::json!({
//         "profile": profile,
//         "wallet_address": wallet_address,
//         "total_asset_value": total_value
//     }))
// }

/// 配置User路由
pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/users")
            .route("/me", web::get().to(get_current_profile))
            .route("/update_profile", web::post().to(update_profile))
            // .route("/wallet/{address}", web::get().to(get_profile_by_wallet)),
    );
}
