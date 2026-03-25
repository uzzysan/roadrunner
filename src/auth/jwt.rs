use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{config::Config, models::user::UserRole};

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

/// Generuje parę tokenów JWT (access + refresh)
///
/// # Arguments
/// * `user_id` - ID użytkownika
/// * `email` - Email użytkownika
/// * `role` - Rola użytkownika
/// * `config` - Konfiguracja zawierająca JWT_SECRET
///
/// # Returns
/// * `TokenPair` - Para tokenów (access + refresh)
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

    // Refresh token - 7 dni
    let refresh_exp = now + Duration::days(7);
    let refresh_claims = Claims {
        sub: user_id,
        email,
        role,
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

/// Dekoduje i waliduje token JWT
///
/// # Arguments
/// * `token` - Token JWT do walidacji
/// * `secret` - Sekret do walidacji
///
/// # Returns
/// * `Claims` - Zdekodowane claims
/// * `Error` - Błąd walidacji
pub fn decode_token(token: &str, secret: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
    let decoding_key = DecodingKey::from_secret(secret.as_bytes());
    let validation = Validation::default();

    let token_data = decode::<Claims>(token, &decoding_key, &validation)?;
    Ok(token_data.claims)
}

/// Odświeża access token używając refresh tokena
///
/// # Arguments
/// * `refresh_token` - Refresh token
/// * `config` - Konfiguracja
///
/// # Returns
/// * `TokenPair` - Nowa para tokenów
pub fn refresh_access_token(
    refresh_token: &str,
    config: &Config,
) -> Result<TokenPair, jsonwebtoken::errors::Error> {
    let claims = decode_token(refresh_token, &config.jwt_secret)?;

    // Generuj nową parę tokenów
    generate_token_pair(claims.sub, claims.email, claims.role, config)
}
