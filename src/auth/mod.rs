pub mod jwt;
pub mod password;

use axum::{
    extract::State,
    http::{Request, StatusCode},
    middleware::Next,
    response::Response,
};
use jsonwebtoken::{decode, DecodingKey, Validation};
use std::sync::Arc;

use crate::config::Config;

#[derive(Debug, Clone)]
pub struct CurrentUser {
    pub user_id: uuid::Uuid,
    pub email: String,
    pub role: crate::models::user::UserRole,
}

pub async fn auth_middleware(
    State(config): State<Arc<Config>>,
    request: Request<axum::body::Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    let auth_header = request
        .headers()
        .get("authorization")
        .and_then(|header| header.to_str().ok());

    let token = match auth_header {
        Some(header) if header.starts_with("Bearer ") => &header[7..],
        _ => return Err(StatusCode::UNAUTHORIZED),
    };

    let validation = Validation::default();
    let decoding_key = DecodingKey::from_secret(config.jwt_secret.as_bytes());

    match decode::<jwt::Claims>(token, &decoding_key, &validation) {
        Ok(token_data) => {
            let claims = token_data.claims;
            let current_user = CurrentUser {
                user_id: claims.sub,
                email: claims.email,
                role: claims.role,
            };
            let mut req = request;
            req.extensions_mut().insert(current_user);
            Ok(next.run(req).await)
        }
        Err(_) => Err(StatusCode::UNAUTHORIZED),
    }
}
