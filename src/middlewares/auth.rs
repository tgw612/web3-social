use actix_web::dev::{Service, ServiceRequest, ServiceResponse, Transform};
use actix_web::error::{ErrorForbidden, ErrorUnauthorized};
use actix_web::http::header::{HeaderName, HeaderValue};
use actix_web::{Error, HttpMessage};
use crate::utils::jwt;
use futures::future::{ok, Ready};
use futures::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use uuid::Uuid;

// 认证中间件
pub struct Auth;

impl Auth {
    pub fn new() -> Self {
        Auth
    }
}

// 用于存储在请求扩展中的用户信息
#[derive(Debug, Clone)]
pub struct AuthenticatedUser {
    pub user_id: String,
    pub wallet_address: String,
    pub wallet_chain: String,
}

// 实现Transform特性
impl<S, B> Transform<S, ServiceRequest> for Auth
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = AuthMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(AuthMiddleware { service })
    }
}

pub struct AuthMiddleware<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for AuthMiddleware<S>
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
        // 从请求头中获取认证令牌
        let token = req
            .headers()
            .get("Authorization")
            .map(|h| h.to_str().unwrap_or_default())
            .unwrap_or_default()
            .trim_start_matches("Bearer ")
            .to_string();

        if token.is_empty() {
            return Box::pin(async move {
                Err(ErrorUnauthorized("Missing authorization token"))
            });
        }

        // 验证JWT令牌
        let user_info: AuthenticatedUser = match jwt::validate_token(&token) {
            Ok(claims) => {
                let user_id: String = match Uuid::parse_str(&claims.sub) {
                    Ok(i) => user_id.to_string(),
                    Err(_) => {
                        return Box::pin(async move {
                            Err(ErrorForbidden("Invalid user ID"))
                        });
                    }
                };

                AuthenticatedUser {
                    user_id,
                    wallet_address: claims.wallet_address,
                    wallet_chain: claims.wallet_chain,
                }
            }
            Err(_) => {
                return Box::pin(async move {
                    Err(ErrorUnauthorized("Invalid authorization token"))
                });
            }
        };

        // 将用户信息添加到请求扩展中，供后续处理使用
        req.extensions_mut().insert(user_info);

        let fut = self.service.call(req);

        Box::pin(async move {
            let res = fut.await?;
            Ok(res)
        })
    }
} 