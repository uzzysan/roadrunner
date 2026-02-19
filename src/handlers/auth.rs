use axum::{
    extract::State,
    http::StatusCode,
    Json,
};
use std::sync::Arc;
use validator::Validate;

use crate::{
    auth::{jwt::generate_token_pair, password::{hash_password, verify_password}},
    config::Config,
    models::user::{CreateUserRequest, LoginRequest, UserResponse, UserRole},
    AppState,
};

#[derive(Debug, serde::Serialize)]
pub struct AuthResponse {
    pub user: UserResponse,
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: i64,
}

#[derive(Debug, serde::Serialize)]
pub struct ErrorResponse {
    pub error: String,
}

pub async fn register(
    State(state): State<AppState>,
    Json(req): Json<CreateUserRequest>,
) -> Result<Json<AuthResponse>, (StatusCode, Json<ErrorResponse>)> {
    if let Err(e) = req.validate() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse { error: e.to_string() }),
        ));
    }

    // TODO: Check if email exists, create user, generate tokens
    
    Err((
        StatusCode::NOT_IMPLEMENTED,
        Json(ErrorResponse { error: "Registration not yet implemented".to_string() }),
    ))
}

pub async fn login(
    State(state): State<AppState>,
    Json(req): Json<LoginRequest>,
) -> Result<Json<AuthResponse>, (StatusCode, Json<ErrorResponse>)> {
    // TODO: Find user, verify password, generate tokens
    
    Err((
        StatusCode::NOT_IMPLEMENTED,
        Json(ErrorResponse { error: "Login not yet implemented".to_string() }),
    ))
}
