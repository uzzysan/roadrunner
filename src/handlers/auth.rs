use axum::{
    extract::State,
    http::StatusCode,
    Json,
};
use sqlx::{PgPool, Row};
use validator::Validate;

use crate::{
    auth::{jwt::generate_token_pair, password::{hash_password, verify_password}},
    models::user::{CreateUserRequest, LoginRequest, UserResponse, UserRole},
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
    State(pool): State<PgPool>,
    Json(req): Json<CreateUserRequest>,
) -> Result<Json<AuthResponse>, (StatusCode, Json<ErrorResponse>)> {
    if let Err(e) = req.validate() {
        return Err((StatusCode::BAD_REQUEST, Json(ErrorResponse { error: e.to_string() })));
    }

    // Check email exists
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users WHERE email = $1")
        .bind(&req.email)
        .fetch_one(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse { error: e.to_string() })))?;

    if count > 0 {
        return Err((StatusCode::CONFLICT, Json(ErrorResponse { error: "Email exists".to_string() })));
    }

    let password_hash = hash_password(&req.password)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse { error: e.to_string() })))?;

    let role_str = "passenger";

    let result = sqlx::query(
        "INSERT INTO users (email, email_hash, password_hash, first_name, last_name, phone, role) 
         VALUES ($1, MD5($1), $2, $3, $4, $5, $6) 
         RETURNING id, email, first_name, last_name"
    )
    .bind(&req.email)
    .bind(&password_hash)
    .bind(&req.first_name)
    .bind(&req.last_name)
    .bind(&req.phone)
    .bind(role_str)
    .fetch_one(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse { error: e.to_string() })))?;

    let user_id: uuid::Uuid = result.get("id");
    let email: String = result.get("email");
    let first_name: String = result.get("first_name");
    let last_name: String = result.get("last_name");

    let user_response = UserResponse {
        id: user_id,
        email: email.clone(),
        first_name,
        last_name,
        phone: req.phone.clone(),
        role: UserRole::Passenger,
        email_verified: false,
        created_at: chrono::Utc::now(),
    };

    let token_pair = generate_token_pair(user_id, email, UserRole::Passenger)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse { error: e.to_string() })))?;

    Ok(Json(AuthResponse {
        user: user_response,
        access_token: token_pair.access_token,
        refresh_token: token_pair.refresh_token,
        expires_in: token_pair.expires_in,
    }))
}

pub async fn login(
    State(pool): State<PgPool>,
    Json(req): Json<LoginRequest>,
) -> Result<Json<AuthResponse>, (StatusCode, Json<ErrorResponse>)> {
    let result = sqlx::query(
        "SELECT id, email, password_hash, first_name, last_name FROM users WHERE email = $1 AND is_active = true"
    )
    .bind(&req.email)
    .fetch_optional(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse { error: e.to_string() })))?;

    let row = match result {
        Some(r) => r,
        None => return Err((StatusCode::UNAUTHORIZED, Json(ErrorResponse { error: "Invalid credentials".to_string() }))),
    };

    let id: uuid::Uuid = row.get("id");
    let email: String = row.get("email");
    let hash: String = row.get("password_hash");
    let first_name: String = row.get("first_name");
    let last_name: String = row.get("last_name");

    if !verify_password(&req.password, &hash).unwrap_or(false) {
        return Err((StatusCode::UNAUTHORIZED, Json(ErrorResponse { error: "Invalid credentials".to_string() })));
    }

    let user_response = UserResponse {
        id,
        email: email.clone(),
        first_name,
        last_name,
        phone: None,
        role: UserRole::Passenger,
        email_verified: false,
        created_at: chrono::Utc::now(),
    };

    let token_pair = generate_token_pair(id, email, UserRole::Passenger)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse { error: e.to_string() })))?;

    Ok(Json(AuthResponse {
        user: user_response,
        access_token: token_pair.access_token,
        refresh_token: token_pair.refresh_token,
        expires_in: token_pair.expires_in,
    }))
}
