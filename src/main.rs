use actix_cors::Cors;
use actix_web::{middleware, web, App, HttpServer};
use dotenv::dotenv;
use log::info;
use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};
use std::env;
use std::sync::Arc;

// 导入rbatis配置
mod config;
use config::rbatis_config;

mod api;
mod blockchain;
mod middlewares;
mod models;
mod services;
mod utils;
pub mod schema;

type DbPool = r2d2::Pool<ConnectionManager<PgConnection>>;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // 初始化环境变量和日志
    dotenv().ok();
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));
    
    // 读取配置
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let host = env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let server_url = format!("{}:{}", host, port);
    
    // 创建数据库连接池
    let manager = ConnectionManager::<PgConnection>::new(database_url.clone());
    let pool = r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to create pool");
        
    // 初始化Rbatis连接池
    let rb = rbatis_config::init_rbatis(&database_url).await;
    
    info!("Starting server at: {}", server_url);
    
    // 初始化Redis连接
    let redis_url = env::var("REDIS_URL").expect("REDIS_URL must be set");
    let redis_client = redis::Client::open(redis_url).expect("Failed to connect to Redis");
    let redis_pool = web::Data::new(redis_client);
    
    // 启动HTTP服务器
    HttpServer::new(move || {
        // 配置CORS
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .max_age(3600);
        
        App::new()
            .wrap(middleware::Logger::default())
            .wrap(cors)
            .app_data(web::Data::new(pool.clone()))
            .app_data(redis_pool.clone())
            .app_data(web::Data::new(rb.clone()))
            // 注册API路由
            .configure(api::user::config)
            .configure(api::asset::config)
            .configure(api::post::config)
            .configure(api::comment::config)
            .configure(api::auth::config)
            // 默认404处理
            .default_service(web::route().to(api::not_found))
    })
    .bind(server_url)?
    .run()
    .await
}