// use crate::services::asset_service::AssetService;
// use crate::utils::error::ServiceError;
// use crate::middlewares::auth::AuthenticatedUser;
// use actix_web::{web, HttpResponse, Responder};
// use serde::{Deserialize, Serialize};
// use std::sync::Arc;

// /// 获取用户资产列表
// pub async fn get_assets(
//     auth_user: AuthenticatedUser,
//     user_service: web::Data<Arc<crate::services::user_service::UserService>>,
//     asset_service: web::Data<Arc<AssetService>>,
// ) -> impl Responder {
//     // 获取用户钱包地址
//     let wallet_address: String = match user_service.get_wallet_address_by_user_id(auth_user.user_id).await {
//         Ok(address) => address,
//         Err(_) => {
//             return HttpResponse::InternalServerError().json(serde_json::json!({
//                 "status": "error",
//                 "message": "获取钱包地址失败"
//             }));
//         }
//     };

//     // 获取用户资产
//     match asset_service.get_user_assets(&wallet_address).await {
//         Ok(assets) => HttpResponse::Ok().json(assets),
//         Err(err) => {
//             HttpResponse::InternalServerError().json(serde_json::json!({
//                 "status": "error",
//                 "message": format!("获取资产失败: {}", err)
//             }))
//         }
//     }
// }

// /// 获取特定钱包地址的资产
// pub async fn get_wallet_assets(
//     path: web::Path<String>,
//     asset_service: web::Data<Arc<AssetService>>,
// ) -> impl Responder {
//     let wallet_address = path.into_inner();
    
//     // 获取指定钱包的资产
//     match asset_service.get_user_assets(&wallet_address).await {
//         Ok(assets) => HttpResponse::Ok().json(assets),
//         Err(err) => {
//             HttpResponse::InternalServerError().json(serde_json::json!({
//                 "status": "error",
//                 "message": format!("获取资产失败: {}", err)
//             }))
//         }
//     }
// }

// /// 获取用户NFT资产
// pub async fn get_nfts(
//     auth_user: AuthenticatedUser,
//     user_service: web::Data<Arc<crate::services::user_service::UserService>>,
//     asset_service: web::Data<Arc<AssetService>>,
// ) -> impl Responder {
//     // 获取用户钱包地址
//     let wallet_address = match user_service.get_wallet_address_by_user_id(auth_user.user_id).await {
//         Ok(address) => address,
//         Err(_) => {
//             return HttpResponse::InternalServerError().json(serde_json::json!({
//                 "status": "error",
//                 "message": "获取钱包地址失败"
//             }));
//         }
//     };

//     // 获取用户NFT
//     match asset_service.get_user_nfts(&wallet_address).await {
//         Ok(nfts) => HttpResponse::Ok().json(nfts),
//         Err(err) => {
//             HttpResponse::InternalServerError().json(serde_json::json!({
//                 "status": "error",
//                 "message": format!("获取NFT资产失败: {}", err)
//             }))
//         }
//     }
// }

// /// 获取指定钱包的NFT资产
// pub async fn get_wallet_nfts(
//     path: web::Path<String>,
//     asset_service: web::Data<Arc<AssetService>>,
// ) -> impl Responder {
//     let wallet_address = path.into_inner();
    
//     // 获取指定钱包的NFT
//     match asset_service.get_user_nfts(&wallet_address).await {
//         Ok(nfts) => HttpResponse::Ok().json(nfts),
//         Err(err) => {
//             HttpResponse::InternalServerError().json(serde_json::json!({
//                 "status": "error",
//                 "message": format!("获取NFT资产失败: {}", err)
//             }))
//         }
//     }
// }

// /// 获取资产总价值
// pub async fn get_total_value(
//     auth_user: AuthenticatedUser,
//     user_service: web::Data<Arc<crate::services::user_service::UserService>>,
//     asset_service: web::Data<Arc<AssetService>>,
// ) -> impl Responder {
//     // 获取用户钱包地址
//     let wallet_address = match user_service.get_wallet_address_by_user_id(auth_user.user_id).await {
//         Ok(address) => address,
//         Err(_) => {
//             return HttpResponse::InternalServerError().json(serde_json::json!({
//                 "status": "error",
//                 "message": "获取钱包地址失败"
//             }));
//         }
//     };

//     // 获取总价值
//     match asset_service.get_total_value(&wallet_address).await {
//         Ok(value) => HttpResponse::Ok().json(serde_json::json!({
//             "total_value": value,
//             "currency": "USD"
//         })),
//         Err(err) => {
//             HttpResponse::InternalServerError().json(serde_json::json!({
//                 "status": "error",
//                 "message": format!("获取资产总价值失败: {}", err)
//             }))
//         }
//     }
// }

// /// 获取指定钱包的资产总价值
// pub async fn get_wallet_total_value(
//     path: web::Path<String>,
//     asset_service: web::Data<Arc<AssetService>>,
// ) -> impl Responder {
//     let wallet_address = path.into_inner();
    
//     // 获取总价值
//     match asset_service.get_total_value(&wallet_address).await {
//         Ok(value) => HttpResponse::Ok().json(serde_json::json!({
//             "total_value": value,
//             "currency": "USD"
//         })),
//         Err(err) => {
//             HttpResponse::InternalServerError().json(serde_json::json!({
//                 "status": "error",
//                 "message": format!("获取资产总价值失败: {}", err)
//             }))
//         }
//     }
// }

// /// 配置Asset路由
// pub fn config(cfg: &mut web::ServiceConfig) {
//     cfg.service(
//         web::scope("/assets")
//             // 当前用户资产相关
//             .route("/me", web::get().to(get_assets))
//             .route("/me/nfts", web::get().to(get_nfts))
//             .route("/me/total", web::get().to(get_total_value))
//             // 查询指定钱包资产
//             .route("/wallet/{address}", web::get().to(get_wallet_assets))
//             .route("/wallet/{address}/nfts", web::get().to(get_wallet_nfts))
//             .route("/wallet/{address}/total", web::get().to(get_wallet_total_value))
//     );
// } 