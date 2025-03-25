use log::info;
use rbatis::RBatis;
use std::sync::Arc;

/// 初始化Rbatis连接池
pub async fn init_rbatis() -> Arc<RBatis> {
    let rb = RBatis::new();
    // MySQL
    // rb.link(rbdc_mysql::driver::MysqlDriver{}, "mysql://root:123456@localhost:3306/test").await.unwrap();
    // PostgreSQL
    rb.link(
        rbdc_pg::driver::PgDriver {},
        "postgres://postgres:123456@localhost:5432/postgres",
    )
    .await
    .unwrap();
    info!("Rbatis initialized successfully");
    Arc::new(rb)
}
