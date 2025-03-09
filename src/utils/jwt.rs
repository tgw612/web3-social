use crate::models::auth::Claims;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use std::env;
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

pub fn generate_token(user_id: Uuid, wallet_address: &str, wallet_chain: &str) -> Result<String, jsonwebtoken::errors::Error> {
    let expiration = env::var("JWT_EXPIRATION")
        .unwrap_or_else(|_| "86400".to_string()) // 默认24小时
        .parse::<usize>()
        .unwrap_or(86400);
    
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs() as usize;
    
    let claims = Claims {
        sub: user_id.to_string(),
        wallet_address: wallet_address.to_string(),
        wallet_chain: wallet_chain.to_string(),
        exp: now + expiration,
        iat: now,
    };
    
    let secret = env::var("JWT_SECRET").expect("JWT_SECRET must be set");
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes())
    )
}

pub fn validate_token(token: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
    let secret = env::var("JWT_SECRET").expect("JWT_SECRET must be set");
    let validation = Validation::default();
    
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &validation
    )?;
    
    Ok(token_data.claims)
} 