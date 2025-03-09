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