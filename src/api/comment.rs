use crate::services::content_service::ContentService;
use crate::services::user_service::UserService;
use crate::utils::error::ServiceError;
use crate::middlewares::auth::AuthenticatedUser;
use actix_web::{web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Deserialize)]
pub struct CreateCommentRequest {
    content: String,
    parent_id: Option<i32>, // 回复的父评论ID，如果是直接评论帖子则为None
}

#[derive(Debug, Deserialize)]
pub struct CommentListQuery {
    page: Option<i32>,
    page_size: Option<i32>,
}

/// 创建评论
pub async fn create_comment(
    path: web::Path<i32>,
    data: web::Json<CreateCommentRequest>,
    auth_user: AuthenticatedUser,
    content_service: web::Data<Arc<ContentService>>,
    user_service: web::Data<Arc<UserService>>,
) -> impl Responder {
    let post_id = path.into_inner();
    
    // 获取用户钱包地址
    let wallet_address = match user_service.get_wallet_address_by_user_id(auth_user.user_id).await {
        Ok(address) => address,
        Err(_) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "status": "error",
                "message": "获取钱包地址失败"
            }));
        }
    };

    // 创建评论
    match content_service.create_comment(
        auth_user.user_id,
        &wallet_address,
        post_id,
        &data.content,
        data.parent_id,
    ).await {
        Ok(comment) => HttpResponse::Created().json(comment),
        Err(err) => {
            match err {
                ServiceError::NotFound(msg) => {
                    HttpResponse::NotFound().json(serde_json::json!({
                        "status": "error",
                        "message": msg
                    }))
                },
                _ => {
                    HttpResponse::InternalServerError().json(serde_json::json!({
                        "status": "error",
                        "message": format!("创建评论失败: {}", err)
                    }))
                }
            }
        }
    }
}

/// 获取帖子的评论列表
pub async fn get_post_comments(
    path: web::Path<i32>,
    query: web::Query<CommentListQuery>,
    content_service: web::Data<Arc<ContentService>>,
    user_service: web::Data<Arc<UserService>>,
) -> impl Responder {
    let post_id = path.into_inner();
    
    // 设置默认分页参数
    let page = query.page.unwrap_or(1);
    let page_size = query.page_size.unwrap_or(20);
    
    // 获取帖子的评论
    let comments = match content_service.get_comments(post_id, page, page_size).await {
        Ok(comments) => comments,
        Err(err) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "status": "error",
                "message": format!("获取评论失败: {}", err)
            }));
        }
    };

    // 为每个评论添加作者信息
    let mut comments_with_authors = Vec::with_capacity(comments.len());
    
    for comment in comments {
        // 获取评论作者信息
        let author = match user_service.get_profile(comment.user_id).await {
            Ok(profile) => profile,
            Err(_) => {
                continue; // 如果获取作者信息失败，跳过该评论
            }
        };
        
        // 组合评论和作者信息
        comments_with_authors.push(serde_json::json!({
            "comment": comment,
            "author": author
        }));
    }

    HttpResponse::Ok().json(comments_with_authors)
}

/// 获取评论的回复列表
pub async fn get_comment_replies(
    path: web::Path<i32>,
    query: web::Query<CommentListQuery>,
    content_service: web::Data<Arc<ContentService>>,
    user_service: web::Data<Arc<UserService>>,
) -> impl Responder {
    let comment_id = path.into_inner();
    
    // 设置默认分页参数
    let page = query.page.unwrap_or(1);
    let page_size = query.page_size.unwrap_or(20);
    
    // 获取评论的回复
    let replies = match content_service.get_comment_replies(comment_id, page, page_size).await {
        Ok(replies) => replies,
        Err(err) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "status": "error",
                "message": format!("获取回复失败: {}", err)
            }));
        }
    };

    // 为每个回复添加作者信息
    let mut replies_with_authors = Vec::with_capacity(replies.len());
    
    for reply in replies {
        // 获取回复作者信息
        let author = match user_service.get_profile(reply.user_id).await {
            Ok(profile) => profile,
            Err(_) => {
                continue; // 如果获取作者信息失败，跳过该回复
            }
        };
        
        // 组合回复和作者信息
        replies_with_authors.push(serde_json::json!({
            "comment": reply,
            "author": author
        }));
    }

    HttpResponse::Ok().json(replies_with_authors)
}

/// 获取评论详情
pub async fn get_comment(
    path: web::Path<i32>,
    content_service: web::Data<Arc<ContentService>>,
    user_service: web::Data<Arc<UserService>>,
) -> impl Responder {
    let comment_id = path.into_inner();
    
    // 获取评论详情
    let comment = match content_service.get_comment(comment_id).await {
        Ok(comment) => comment,
        Err(err) => {
            return match err {
                ServiceError::NotFound(_) => {
                    HttpResponse::NotFound().json(serde_json::json!({
                        "status": "error",
                        "message": "评论不存在"
                    }))
                },
                _ => {
                    HttpResponse::InternalServerError().json(serde_json::json!({
                        "status": "error",
                        "message": format!("获取评论失败: {}", err)
                    }))
                }
            }
        }
    };

    // 获取评论作者信息
    let author = match user_service.get_profile(comment.user_id).await {
        Ok(profile) => profile,
        Err(_) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "status": "error",
                "message": "获取作者信息失败"
            }));
        }
    };

    HttpResponse::Ok().json(serde_json::json!({
        "comment": comment,
        "author": author
    }))
}

/// 配置Comment路由
pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/comments")
            // 获取评论详情
            .route("/{comment_id}", web::get().to(get_comment))
            // 获取评论的回复
            .route("/{comment_id}/replies", web::get().to(get_comment_replies))
    );
    
    // 帖子评论相关路由，放在posts命名空间下
    cfg.service(
        web::scope("/posts")
            // 创建评论
            .route("/{post_id}/comments", web::post().to(create_comment))
            // 获取帖子的评论
            .route("/{post_id}/comments", web::get().to(get_post_comments))
    );
} 