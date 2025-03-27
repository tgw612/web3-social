pub mod auth;
pub mod user;
// pub mod asset;
// pub mod post;
// pub mod comment;

use actix_web::{HttpResponse, web};

// 处理404的默认处理函数
pub async fn not_found() -> HttpResponse {
    HttpResponse::NotFound().json(serde_json::json!({
        "status": "error",
        "message": "Resource not found"
    }))
}

// API路由配置
// 在 main.rs 中实际调用路由配置
pub fn config(cfg: &mut web::ServiceConfig) {
    // 添加其他模块的路由配置
    user::config(cfg);
    auth::config(cfg);
    // ... 其他模块配置
}