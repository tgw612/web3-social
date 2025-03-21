use rbatis::rbatis::Rbatis;
use std::sync::Arc;
use log::info;

/// 初始化Rbatis连接池
pub async fn init_rbatis(database_url: &str) -> Arc<Rbatis> {
    let rb = Rbatis::new();
    rb.init_opt(database_url, rbdc_pg::driver::PgDriver {}, rbatis::pool::Pool::default())
        .await
        .expect("Failed to connect to database");
    
    info!("Rbatis initialized successfully");
    Arc::new(rb)
}