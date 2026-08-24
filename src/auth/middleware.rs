//! Axum extractor for authenticated requests.
//!
//! `handlers/payments.rs` and `handlers/tickets.rs` were written against
//! `auth::middleware::AuthUser` used as an extractor (`AuthUser(user): AuthUser`), but this
//! module never existed in the repository — compiling either handler failed with
//! `error[E0432]: unresolved import` (found during the 2026-08-24 status review). This fills
//! that gap: `AuthUser` decodes and validates the JWT straight from the `Authorization: Bearer
//! <token>` header, independent of the separate `auth_middleware`/`CurrentUser` tower-middleware
//! pattern already implemented in `auth::mod` (which nothing currently `.layer(...)`s onto the
//! router — see `docs/status-log.md`/session notes for the wider status picture).

use axum::{
    async_trait,
    extract::{FromRef, FromRequestParts},
    http::{header, request::Parts},
};

use crate::{
    auth::jwt::{self, Claims},
    errors::AppError,
    state::AppState,
};

/// Extracts and validates the caller's JWT, giving handlers the decoded [`Claims`].
pub struct AuthUser(pub Claims);

#[async_trait]
impl<S> FromRequestParts<S> for AuthUser
where
    AppState: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let app_state = AppState::from_ref(state);

        let header_value = parts
            .headers
            .get(header::AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
            .ok_or_else(|| AppError::Unauthorized("Missing Authorization header".to_string()))?;

        let token = header_value
            .strip_prefix("Bearer ")
            .ok_or_else(|| AppError::Unauthorized("Invalid Authorization header".to_string()))?;

        let claims = jwt::decode_token(token, &app_state.config.jwt_secret)
            .map_err(|_| AppError::Unauthorized("Invalid or expired token".to_string()))?;

        Ok(AuthUser(claims))
    }
}
