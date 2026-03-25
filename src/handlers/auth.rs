use axum::{
    extract::State,
    Json,
};
use validator::Validate;

use crate::{
    auth::{jwt::generate_token_pair, password::{hash_password, verify_password}},
    errors::{AppError, AppResult},
    models::user::{CreateUserRequest, LoginRequest, UserResponse, UserRole},
    state::AppState,
};

#[derive(Debug, serde::Serialize)]
pub struct AuthResponse {
    pub user: UserResponse,
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: i64,
}

pub async fn register(
    State(state): State<AppState>,
    Json(req): Json<CreateUserRequest>,
) -> AppResult<Json<AuthResponse>> {
    // Walidacja danych wejściowych
    req.validate()?;

    // Sprawdź czy email istnieje
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users WHERE email = $1")
        .bind(&req.email)
        .fetch_one(&state.db)
        .await?;

    if count > 0 {
        return Err(AppError::Conflict("Email already registered".to_string()));
    }

    // Hash hasła
    let password_hash = hash_password(&req.password)
        .map_err(|e| AppError::Internal(format!("Password hashing failed: {}", e)))?;

    // Utwórz użytkownika
    let user = sqlx::query_as!(
        crate::models::user::User,
        r#"
        INSERT INTO users (email, password_hash, first_name, last_name, phone, role)
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING id, email, password_hash, first_name, last_name, phone, role as "role: UserRole", 
                  mfa_enabled, mfa_secret, created_at, updated_at
        "#,
        req.email,
        password_hash,
        req.first_name,
        req.last_name,
        req.phone,
        UserRole::Passenger as UserRole,
    )
    .fetch_one(&state.db)
    .await?;

    // Generuj tokeny JWT
    let token_pair = generate_token_pair(user.id, user.role.clone(), &state.config)
        .map_err(|e| AppError::Internal(format!("Token generation failed: {}", e)))?;

    Ok(Json(AuthResponse {
        user: UserResponse::from(user),
        access_token: token_pair.access_token,
        refresh_token: token_pair.refresh_token,
        expires_in: token_pair.expires_in,
    }))
}

pub async fn login(
    State(state): State<AppState>,
    Json(req): Json<LoginRequest>,
) -> AppResult<Json<AuthResponse>> {
    // Walidacja danych wejściowych
    req.validate()?;

    // Znajdź użytkownika po emailu
    let user = sqlx::query_as!(
        crate::models::user::User,
        r#"
        SELECT id, email, password_hash, first_name, last_name, phone, 
               role as "role: UserRole", mfa_enabled, mfa_secret, created_at, updated_at
        FROM users 
        WHERE email = $1
        "#,
        req.email
    )
    .fetch_optional(&state.db)
    .await?;

    let user = user.ok_or_else(|| AppError::Unauthorized("Invalid credentials".to_string()))?;

    // Weryfikacja hasła
    let valid = verify_password(&req.password, &user.password_hash)
        .map_err(|_| AppError::Unauthorized("Invalid credentials".to_string()))?;

    if !valid {
        return Err(AppError::Unauthorized("Invalid credentials".to_string()));
    }

    // TODO: Sprawdź MFA jeśli włączone
    if user.mfa_enabled {
        // MFA flow - zwróć tymczasowy token lub wymagaj kodu MFA
        // Na razie pomijamy, będzie zaimplementowane w Fazie 1
    }

    // Generuj tokeny JWT
    let token_pair = generate_token_pair(user.id, user.role.clone(), &state.config)
        .map_err(|e| AppError::Internal(format!("Token generation failed: {}", e)))?;

    Ok(Json(AuthResponse {
        user: UserResponse::from(user),
        access_token: token_pair.access_token,
        refresh_token: token_pair.refresh_token,
        expires_in: token_pair.expires_in,
    }))
}
