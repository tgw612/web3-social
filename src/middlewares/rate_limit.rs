use actix_web::dev::{Service, ServiceRequest, ServiceResponse, Transform};
use actix_web::http::header::HeaderMap;
use actix_web::Error;
use futures::future::{ok, Ready};
use futures::Future;
use redis::{AsyncCommands, Client};
use std::pin::Pin;
use std::rc::Rc;
use std::task::{Context, Poll};

// 速率限制中间件
pub struct RateLimit {
    redis_client: Rc<Client>,
    requests_per_minute: usize, // 每分钟允许的请求数
}

impl RateLimit {
    pub fn new(redis_client: Client, requests_per_minute: usize) -> Self {
        RateLimit {
            redis_client: Rc::new(redis_client),
            requests_per_minute,
        }
    }
}

// 实现Transform特性
impl<S, B> Transform<S, ServiceRequest> for RateLimit
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = RateLimitMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(RateLimitMiddleware {
            service,
            redis_client: self.redis_client.clone(),
            requests_per_minute: self.requests_per_minute,
        })
    }
}

pub struct RateLimitMiddleware<S> {
    service: S,
    redis_client: Rc<Client>,
    requests_per_minute: usize,
}

impl<S, B> Service<ServiceRequest> for RateLimitMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    fn poll_ready(&self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        // 获取客户端IP地址
        let client_ip = get_client_ip(&req);
        let key = format!("rate_limit:{}", client_ip);
        let redis_client = self.redis_client.clone();
        let requests_per_minute = self.requests_per_minute;
        
        // 调用原始服务
        let fut = self.service.call(req);
        
        Box::pin(async move {
            // 获取Redis连接
            let mut con = redis_client.get_async_connection().await.map_err(|e| {
                log::error!("Failed to get Redis connection: {}", e);
                actix_web::error::ErrorInternalServerError("Rate limit service error")
            })?;
            
            // 检查是否超过速率限制
            let count: Option<usize> = con.get(&key).await.unwrap_or(None);
            
            if let Some(count) = count {
                if count >= requests_per_minute {
                    return Err(actix_web::error::ErrorTooManyRequests("Rate limit exceeded"));
                }
            }
            
            // 增加计数器
            let _: () = con.incr(&key, 1).await.map_err(|e| {
                log::error!("Failed to increment rate limit counter: {}", e);
                actix_web::error::ErrorInternalServerError("Rate limit service error")
            })?;
            
            // 设置过期时间（如果是新键）
            let _: () = con.expire(&key, 60).await.map_err(|e| {
                log::error!("Failed to set expiry for rate limit key: {}", e);
                actix_web::error::ErrorInternalServerError("Rate limit service error")
            })?;
            
            // 继续处理请求
            fut.await
        })
    }
}

// 获取客户端IP地址
fn get_client_ip(req: &ServiceRequest) -> String {
    let headers = req.headers();
    
    // 尝试从X-Forwarded-For头获取真实IP
    if let Some(ip) = get_ip_from_headers(headers, "x-forwarded-for") {
        return ip;
    }
    
    // 尝试从X-Real-IP头获取
    if let Some(ip) = get_ip_from_headers(headers, "x-real-ip") {
        return ip;
    }
    
    // 回退到连接信息
    req.connection_info().peer_addr()
        .unwrap_or("unknown")
        .to_string()
}

// 从HTTP头中提取IP
fn get_ip_from_headers(headers: &HeaderMap, header_name: &str) -> Option<String> {
    headers.get(header_name)
        .and_then(|h| h.to_str().ok())
        .map(|ip_list| {
            // X-Forwarded-For可能包含多个IP，取第一个（最初的客户端）
            ip_list.split(',')
                .next()
                .unwrap_or("unknown")
                .trim()
                .to_string()
        })
} 