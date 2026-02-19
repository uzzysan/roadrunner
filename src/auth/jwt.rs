use chrono::{Duration, Utc};
use jsonwebtoken::{encode, EncodingKey, Header};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::config::Config;
use crate::models::user::UserRole;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: Uuid,
    pub email: String,
    pub role: UserRole,
    pub exp: i64,
    pub iat: i64,
}

#[derive(Debug, Serialize)]
pub struct TokenPair {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: i64,
}

pub fn generate_token_pair(
    user_id: Uuid,
    email: String,
    role: UserRole,
    config: &Config,
) -> Result<TokenPair, jsonwebtoken::errors::Error> {
    let now = Utc::now();
    let exp = now + Duration::seconds(config.jwt_expiration);
    
    let claims = Claims {
        sub: user_id,
        email: email.clone(),
        role: role.clone(),
        exp: exp.timestamp(),
        iat: now.timestamp(),
    };
    
    let header = Header::default();
    let encoding_key = EncodingKey::from_secret(config.jwt_secret.as_bytes());
    
    let access_token = encode(&header, &claims, &encoding_key)?;
    
    // Refresh token expires in 7 days
    let refresh_exp = now + Duration::days(7);
    let refresh_claims = Claims {
        sub: user_id,
        email,
        role: role.clone(),
        exp: refresh_exp.timestamp(),
        iat: now.timestamp(),
    };
    let refresh_token = encode(&header, &refresh_claims, &encoding_key)?;
    
    Ok(TokenPair {
        access_token,
        refresh_token,
        expires_in: config.jwt_expiration,
    })
}
