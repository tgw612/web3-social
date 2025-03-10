use crate::services::content_service::ContentService;
use crate::services::user_service::UserService;
use crate::services::storage_service::StorageService;
use crate::utils::error::ServiceError;
use crate::middlewares::auth::AuthenticatedUser;
use actix_web::{web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use base64;

#[derive(Debug, Deserialize)]
pub struct CreatePostRequest {
    content: String,
    image_data: Option<String>, // Base64编码的图片数据
    tags: Vec<String>,
    tx_hash: Option<String>,    // 可选的交易哈希，用于验证投资操作
}

#[derive(Debug, Deserialize)]
pub struct PostListQuery {
    page: Option<i32>,
    page_size: Option<i32>,
    sort_by: Option<String>, // "hot" 或 "time"
    tag: Option<String>,
}

/// 创建新帖子
pub async fn create_post(
    auth_user: AuthenticatedUser,
    data: web::Json<CreatePostRequest>,
    content_service: web::Data<Arc<ContentService>>,
    user_service: web::Data<Arc<UserService>>,
) -> impl Responder {
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

    // 处理图片数据
    let image_data = match &data.image_data {
        Some(base64_data) => {
            // 解码Base64图片数据
            match base64::decode(base64_data.replace("data:image/jpeg;base64,", "")
                                           .replace("data:image/png;base64,", "")) {
                Ok(decoded) => Some(decoded),
                Err(_) => {
                    return HttpResponse::BadRequest().json(serde_json::json!({
                        "status": "error",
                        "message": "图片数据格式错误"
                    }));
                }
            }
        },
        None => None,
    };

    // 创建帖子
    match content_service.create_post(
        auth_user.user_id,
        &wallet_address,
        &data.content,
        image_data,
        data.tags.clone(),
        data.tx_hash.clone(),
    ).await {
        Ok(post) => HttpResponse::Created().json(post),
        Err(err) => {
            HttpResponse::InternalServerError().json(serde_json::json!({
                "status": "error",
                "message": format!("创建帖子失败: {}", err)
            }))
        }
    }
}

/// 获取帖子列表
pub async fn get_posts(
    query: web::Query<PostListQuery>,
    content_service: web::Data<Arc<ContentService>>,
) -> impl Responder {
    // 设置默认分页参数
    let page = query.page.unwrap_or(1);
    let page_size = query.page_size.unwrap_or(20);
    
    // 根据排序方式获取帖子
    let posts = match query.sort_by.as_deref() {
        Some("hot") | None => {
            // 默认按热度排序
            content_service.get_posts_by_hot(page, page_size).await
        },
        Some("time") => {
            // 按时间排序
            content_service.get_posts_by_time(page, page_size).await
        },
        Some(_) => {
            // 不支持的排序方式
            return HttpResponse::BadRequest().json(serde_json::json!({
                "status": "error",
                "message": "不支持的排序方式"
            }));
        }
    };

    // 返回帖子列表
    match posts {
        Ok(posts) => HttpResponse::Ok().json(posts),
        Err(err) => {
            HttpResponse::InternalServerError().json(serde_json::json!({
                "status": "error",
                "message": format!("获取帖子列表失败: {}", err)
            }))
        }
    }
}

/// 根据标签获取帖子
pub async fn get_posts_by_tag(
    path: web::Path<String>,
    query: web::Query<PostListQuery>,
    content_service: web::Data<Arc<ContentService>>,
) -> impl Responder {
    let tag = path.into_inner();
    
    // 设置默认分页参数
    let page = query.page.unwrap_or(1);
    let page_size = query.page_size.unwrap_or(20);
    
    // 获取指定标签的帖子
    match content_service.get_posts_by_tag(&tag, page, page_size).await {
        Ok(posts) => HttpResponse::Ok().json(posts),
        Err(err) => {
            HttpResponse::InternalServerError().json(serde_json::json!({
                "status": "error",
                "message": format!("获取帖子列表失败: {}", err)
            }))
        }
    }
}

/// 获取用户发布的帖子
pub async fn get_user_posts(
    path: web::Path<i32>,
    query: web::Query<PostListQuery>,
    content_service: web::Data<Arc<ContentService>>,
) -> impl Responder {
    let user_id = path.into_inner();
    
    // 设置默认分页参数
    let page = query.page.unwrap_or(1);
    let page_size = query.page_size.unwrap_or(20);
    
    // 获取用户发布的帖子
    match content_service.get_user_posts(user_id, page, page_size).await {
        Ok(posts) => HttpResponse::Ok().json(posts),
        Err(err) => {
            HttpResponse::InternalServerError().json(serde_json::json!({
                "status": "error",
                "message": format!("获取用户帖子失败: {}", err)
            }))
        }
    }
}

/// 获取帖子详情
pub async fn get_post_detail(
    path: web::Path<i32>,
    content_service: web::Data<Arc<ContentService>>,
    user_service: web::Data<Arc<UserService>>,
    storage_service: web::Data<Arc<StorageService>>,
    auth_user: Option<AuthenticatedUser>,
) -> impl Responder {
    let post_id = path.into_inner();
    
    // 获取帖子详情
    let post = match content_service.get_post(post_id).await {
        Ok(post) => post,
        Err(err) => {
            return match err {
                ServiceError::NotFound(_) => {
                    HttpResponse::NotFound().json(serde_json::json!({
                        "status": "error",
                        "message": "帖子不存在"
                    }))
                },
                _ => {
                    HttpResponse::InternalServerError().json(serde_json::json!({
                        "status": "error",
                        "message": format!("获取帖子失败: {}", err)
                    }))
                }
            }
        }
    };

    // 获取帖子作者信息
    let author_profile = match user_service.get_profile(post.user_id).await {
        Ok(profile) => profile,
        Err(_) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "status": "error",
                "message": "获取作者信息失败"
            }));
        }
    };

    // 获取帖子点赞数
    let likes_count = match content_service.get_post_likes_count(post_id).await {
        Ok(count) => count,
        Err(_) => 0, // 如果获取失败，默认为0
    };

    // 检查当前用户是否已点赞（如果有登录用户）
    let has_liked = if let Some(auth_user) = &auth_user {
        match content_service.has_user_liked(auth_user.user_id, post_id).await {
            Ok(liked) => liked,
            Err(_) => false,
        }
    } else {
        false
    };

    // 如果帖子有图片，生成URL
    let image_url = if let Some(cid) = &post.image_ipfs_cid {
        Some(storage_service.get_ipfs_url(cid))
    } else {
        None
    };

    // 构建响应
    HttpResponse::Ok().json(serde_json::json!({
        "post": post,
        "author": author_profile,
        "likes_count": likes_count,
        "has_liked": has_liked,
        "image_url": image_url
    }))
}

/// 点赞帖子
pub async fn like_post(
    path: web::Path<i32>,
    auth_user: AuthenticatedUser,
    content_service: web::Data<Arc<ContentService>>,
) -> impl Responder {
    let post_id = path.into_inner();
    
    // 点赞帖子
    match content_service.like_post(auth_user.user_id, post_id).await {
        Ok(_) => HttpResponse::Ok().json(serde_json::json!({
            "status": "success",
            "message": "点赞成功"
        })),
        Err(err) => {
            match err {
                ServiceError::BadRequest(msg) => {
                    HttpResponse::BadRequest().json(serde_json::json!({
                        "status": "error",
                        "message": msg
                    }))
                },
                _ => {
                    HttpResponse::InternalServerError().json(serde_json::json!({
                        "status": "error",
                        "message": format!("点赞失败: {}", err)
                    }))
                }
            }
        }
    }
}

/// 取消点赞
pub async fn unlike_post(
    path: web::Path<i32>,
    auth_user: AuthenticatedUser,
    content_service: web::Data<Arc<ContentService>>,
) -> impl Responder {
    let post_id = path.into_inner();
    
    // 取消点赞
    match content_service.unlike_post(auth_user.user_id, post_id).await {
        Ok(_) => HttpResponse::Ok().json(serde_json::json!({
            "status": "success",
            "message": "取消点赞成功"
        })),
        Err(err) => {
            HttpResponse::InternalServerError().json(serde_json::json!({
                "status": "error",
                "message": format!("取消点赞失败: {}", err)
            }))
        }
    }
}

/// 获取热门标签
pub async fn get_hot_tags(
    content_service: web::Data<Arc<ContentService>>,
) -> impl Responder {
    // 获取热门标签（默认返回前10个）
    match content_service.get_hot_tags(10).await {
        Ok(tags) => HttpResponse::Ok().json(tags),
        Err(err) => {
            HttpResponse::InternalServerError().json(serde_json::json!({
                "status": "error",
                "message": format!("获取热门标签失败: {}", err)
            }))
        }
    }
}

/// 搜索帖子
pub async fn search_posts(
    query: web::Query<PostListQuery>,
    search_query: web::Path<String>,
    content_service: web::Data<Arc<ContentService>>,
) -> impl Responder {
    let search_term = search_query.into_inner();
    
    // 设置默认分页参数
    let page = query.page.unwrap_or(1);
    let page_size = query.page_size.unwrap_or(20);
    
    // 搜索帖子
    match content_service.search_posts(&search_term, page, page_size).await {
        Ok(posts) => HttpResponse::Ok().json(posts),
        Err(err) => {
            HttpResponse::InternalServerError().json(serde_json::json!({
                "status": "error",
                "message": format!("搜索帖子失败: {}", err)
            }))
        }
    }
}

/// 配置Post路由
pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/posts")
            // 帖子列表和创建
            .route("", web::get().to(get_posts))
            .route("", web::post().to(create_post))
            // 帖子详情、点赞和取消点赞
            .route("/{post_id}", web::get().to(get_post_detail))
            .route("/{post_id}/like", web::post().to(like_post))
            .route("/{post_id}/unlike", web::post().to(unlike_post))
            // 标签相关
            .route("/tags", web::get().to(get_hot_tags))
            .route("/tag/{tag}", web::get().to(get_posts_by_tag))
            // 用户帖子
            .route("/user/{user_id}", web::get().to(get_user_posts))
            // 搜索
            .route("/search/{query}", web::get().to(search_posts))
    );
} 