pub mod auth;
pub mod user;
pub mod asset;
pub mod post;
pub mod comment;

use actix_web::{HttpResponse, web};

// 处理404的默认处理函数
pub async fn not_found() -> HttpResponse {
    HttpResponse::NotFound().json(serde_json::json!({
        "status": "error",
        "message": "Resource not found"
    }))
}

// API路由配置
pub fn config(cfg: &mut web::ServiceConfig) {
    // 配置所有API路由
    auth::config(cfg);
    user::config(cfg);
    asset::config(cfg);
    post::config(cfg);
    comment::config(cfg);
    
    // 添加默认404处理
    cfg.default_service(web::route().to(not_found));
} 