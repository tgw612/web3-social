use crate::models::auth::Claims;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use std::env;
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;
use crate::utils::error::ServiceError;
use serde::{Serialize, Deserialize};
use chrono;


pub fn generate_token(user_id: Uuid, wallet_address: &str) -> Result<String, ServiceError> {
    let expiration = chrono::Utc::now()
        .checked_add_signed(chrono::Duration::hours(24))
        .expect("valid timestamp")
        .timestamp();

    let claims: Claims = Claims {
        sub: user_id.to_string(),
        wallet_address: wallet_address.to_string(),
        exp: expiration as usize,
        iat: chrono::Utc::now().timestamp() as usize,
        wallet_chain: "ETH".to_string(),
    };

    let secret: String = env::var("JWT_SECRET").map_err(|_| ServiceError::InternalServerError)?;
    let key = EncodingKey::from_secret(secret.as_bytes());
    
    encode(&Header::default(), &claims, &key)
        .map_err(|_| ServiceError::InternalServerError)
}

pub fn validate_token(token: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
    let secret: String = env::var("JWT_SECRET").expect("JWT_SECRET must be set");
    let validation: Validation = Validation::default();
    
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &validation
    )?;
    
    Ok(token_data.claims)
} 