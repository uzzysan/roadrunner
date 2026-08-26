pub mod jwt;
pub mod mfa;
pub mod middleware;
pub mod password;

use axum::{
    extract::State,
    http::{Request, StatusCode},
    middleware::Next,
    response::Response,
};
use jsonwebtoken::{decode, DecodingKey, Validation};
use std::sync::Arc;

use crate::state::AppState;

#[derive(Debug, Clone)]
pub struct CurrentUser {
    pub user_id: uuid::Uuid,
    pub email: String,
    pub role: crate::models::user::UserRole,
}

/// Middleware autentykacji - weryfikuje JWT token
///
/// # Arguments
/// * `State(state)` - Stan aplikacji
/// * `request` - HTTP request
/// * `next` - Next middleware/handler
///
/// # Returns
/// * `Result<Response, StatusCode>` - Response lub 401 Unauthorized
pub async fn auth_middleware(
    State(state): State<Arc<AppState>>,
    mut request: Request<axum::body::Body>,
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
    let decoding_key = DecodingKey::from_secret(state.config.jwt_secret.as_bytes());

    match decode::<jwt::Claims>(token, &decoding_key, &validation) {
        Ok(token_data) => {
            let claims = token_data.claims;
            let current_user = CurrentUser {
                user_id: claims.sub,
                email: claims.email,
                role: claims.role,
            };
            request.extensions_mut().insert(current_user);
            Ok(next.run(request).await)
        }
        Err(_) => Err(StatusCode::UNAUTHORIZED),
    }
}

/// Middleware wymagający konkretnej roli
///
/// # Type Parameters
/// * `F` - Handler function
/// * `Roles` - Lista dozwolonych ról
///
/// # Example
/// ```rust
/// let admin_routes = Router::new()
///     .route("/admin/users", get(list_users))
///     .layer(middleware::from_fn_with_state(
///         state.clone(),
///         require_role![UserRole::Admin],
///     ));
/// ```
pub async fn require_role(
    State(_state): State<Arc<AppState>>,
    request: Request<axum::body::Body>,
    next: Next,
    allowed_roles: Vec<crate::models::user::UserRole>,
) -> Result<Response, StatusCode> {
    let current_user = request
        .extensions()
        .get::<CurrentUser>()
        .ok_or(StatusCode::UNAUTHORIZED)?;

    if !allowed_roles.contains(&current_user.role) {
        return Err(StatusCode::FORBIDDEN);
    }

    Ok(next.run(request).await)
}
