use axum::{extract::State, Json};
use validator::Validate;

use crate::{
    auth::{
        jwt::generate_token_pair,
        password::{hash_password, verify_password},
    },
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
        RETURNING id, email, email_hash, password_hash, first_name, last_name, phone, role as "role: UserRole", 
                  mfa_enabled, mfa_secret, email_verified, is_active, created_at, updated_at
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
    let token_pair = generate_token_pair(
        user.id,
        user.email.clone(),
        user.role.clone(),
        &state.config,
    )
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
        SELECT id, email, email_hash, password_hash, first_name, last_name, phone, 
               role as "role: UserRole", mfa_enabled, mfa_secret, email_verified, is_active, created_at, updated_at
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
    let token_pair = generate_token_pair(
        user.id,
        user.email.clone(),
        user.role.clone(),
        &state.config,
    )
    .map_err(|e| AppError::Internal(format!("Token generation failed: {}", e)))?;

    Ok(Json(AuthResponse {
        user: UserResponse::from(user),
        access_token: token_pair.access_token,
        refresh_token: token_pair.refresh_token,
        expires_in: token_pair.expires_in,
    }))
}

// ==================== MFA ENDPOINTS ====================

#[derive(Debug, serde::Deserialize)]
pub struct SetupMfaRequest {
    pub user_id: uuid::Uuid,
}

#[derive(Debug, serde::Serialize)]
pub struct SetupMfaResponse {
    pub secret: String,
    pub qr_code: String,
    pub qr_url: String,
}

/// Inicjalizacja MFA - generuje sekret i QR code
///
/// # Endpoint
/// POST /auth/mfa/setup
///
/// # Response
/// ```json
/// {
///   "secret": "base32secret...",
///   "qr_code": "data:image/svg+xml;base64,...",
///   "qr_url": "otpauth://totp/..."
/// }
/// ```
pub async fn setup_mfa(
    State(state): State<AppState>,
    Json(req): Json<SetupMfaRequest>,
) -> AppResult<Json<SetupMfaResponse>> {
    // Pobierz użytkownika
    let user = sqlx::query_as!(
        crate::models::user::User,
        r#"
        SELECT id, email, email_hash, password_hash, first_name, last_name, phone, 
               role as "role: UserRole", mfa_enabled, mfa_secret, email_verified, is_active, created_at, updated_at
        FROM users 
        WHERE id = $1
        "#,
        req.user_id
    )
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

    // Sprawdź czy MFA nie jest już włączone
    if user.mfa_enabled {
        return Err(AppError::Conflict("MFA is already enabled".to_string()));
    }

    // Generuj sekret TOTP
    let (secret, qr_url) = crate::auth::mfa::generate_totp_secret(&user.email)?;

    // Generuj QR code
    let qr_code = crate::auth::mfa::generate_qr_code(&qr_url)?;

    // Zapisz sekret w bazie (ale nie aktywuj jeszcze MFA)
    sqlx::query!(
        "UPDATE users SET mfa_secret = $1 WHERE id = $2",
        secret,
        req.user_id
    )
    .execute(&state.db)
    .await?;

    Ok(Json(SetupMfaResponse {
        secret,
        qr_code,
        qr_url,
    }))
}

#[derive(Debug, serde::Deserialize)]
pub struct VerifyMfaSetupRequest {
    pub user_id: uuid::Uuid,
    pub code: String,
}

#[derive(Debug, serde::Serialize)]
pub struct VerifyMfaSetupResponse {
    pub success: bool,
    pub message: String,
}

/// Weryfikuje kod MFA i aktywuje MFA dla użytkownika
///
/// # Endpoint
/// POST /auth/mfa/verify-setup
///
/// # Response
/// ```json
/// {
///   "success": true,
///   "message": "MFA enabled successfully"
/// }
/// ```
pub async fn verify_mfa_setup(
    State(state): State<AppState>,
    Json(req): Json<VerifyMfaSetupRequest>,
) -> AppResult<Json<VerifyMfaSetupResponse>> {
    // Pobierz użytkownika z sekretem MFA
    let user = sqlx::query!(
        "SELECT id, mfa_secret FROM users WHERE id = $1",
        req.user_id
    )
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

    let secret = user
        .mfa_secret
        .ok_or_else(|| AppError::BadRequest("MFA not initialized".to_string()))?;

    // Weryfikuj kod
    let is_valid = crate::auth::mfa::verify_totp(&secret, &req.code)?;

    if !is_valid {
        return Ok(Json(VerifyMfaSetupResponse {
            success: false,
            message: "Invalid MFA code".to_string(),
        }));
    }

    // Aktywuj MFA
    sqlx::query!(
        "UPDATE users SET mfa_enabled = true WHERE id = $1",
        req.user_id
    )
    .execute(&state.db)
    .await?;

    Ok(Json(VerifyMfaSetupResponse {
        success: true,
        message: "MFA enabled successfully".to_string(),
    }))
}

#[derive(Debug, serde::Deserialize)]
pub struct VerifyMfaLoginRequest {
    pub temp_token: String,
    pub code: String,
}

#[derive(Debug, serde::Serialize)]
pub struct VerifyMfaLoginResponse {
    pub user: UserResponse,
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: i64,
}

/// Weryfikuje kod MFA podczas logowania
///
/// # Endpoint
/// POST /auth/mfa/verify-login
///
/// # Response
/// ```json
/// {
///   "user": { ... },
///   "access_token": "eyJ...",
///   "refresh_token": "eyJ...",
///   "expires_in": 86400
/// }
/// ```
pub async fn verify_mfa_login(
    State(state): State<AppState>,
    Json(req): Json<VerifyMfaLoginRequest>,
) -> AppResult<Json<VerifyMfaLoginResponse>> {
    // Dekoduj tymczasowy token (zawiera user_id)
    let claims = crate::auth::jwt::decode_token(&req.temp_token, &state.config.jwt_secret)
        .map_err(|_| AppError::Unauthorized("Invalid temp token".to_string()))?;

    // Pobierz użytkownika
    let user = sqlx::query_as!(
        crate::models::user::User,
        r#"
        SELECT id, email, email_hash, password_hash, first_name, last_name, phone, 
               role as "role: UserRole", mfa_enabled, mfa_secret, email_verified, is_active, created_at, updated_at
        FROM users 
        WHERE id = $1
        "#,
        claims.sub
    )
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

    // Sprawdź czy MFA jest włączone
    if !user.mfa_enabled {
        return Err(AppError::BadRequest("MFA is not enabled".to_string()));
    }

    let secret = user
        .mfa_secret
        .clone()
        .ok_or_else(|| AppError::Internal("MFA secret not found".to_string()))?;

    // Weryfikuj kod MFA
    let is_valid = crate::auth::mfa::verify_totp(&secret, &req.code)?;

    if !is_valid {
        return Err(AppError::Unauthorized("Invalid MFA code".to_string()));
    }

    // Generuj pełne tokeny
    let token_pair = generate_token_pair(
        user.id,
        user.email.clone(),
        user.role.clone(),
        &state.config,
    )
    .map_err(|e| AppError::Internal(format!("Token generation failed: {}", e)))?;

    Ok(Json(VerifyMfaLoginResponse {
        user: UserResponse::from(user),
        access_token: token_pair.access_token,
        refresh_token: token_pair.refresh_token,
        expires_in: token_pair.expires_in,
    }))
}

#[derive(Debug, serde::Deserialize)]
pub struct DisableMfaRequest {
    pub user_id: uuid::Uuid,
    pub password: String,
}

#[derive(Debug, serde::Serialize)]
pub struct DisableMfaResponse {
    pub success: bool,
    pub message: String,
}

/// Wyłącza MFA dla użytkownika (wymaga hasła)
///
/// # Endpoint
/// POST /auth/mfa/disable
///
/// # Response
/// ```json
/// {
///   "success": true,
///   "message": "MFA disabled successfully"
/// }
/// ```
pub async fn disable_mfa(
    State(state): State<AppState>,
    Json(req): Json<DisableMfaRequest>,
) -> AppResult<Json<DisableMfaResponse>> {
    // Pobierz użytkownika
    let user = sqlx::query_as!(
        crate::models::user::User,
        r#"
        SELECT id, email, email_hash, password_hash, first_name, last_name, phone, 
               role as "role: UserRole", mfa_enabled, mfa_secret, email_verified, is_active, created_at, updated_at
        FROM users 
        WHERE id = $1
        "#,
        req.user_id
    )
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

    // Weryfikuj hasło
    let valid = verify_password(&req.password, &user.password_hash)
        .map_err(|_| AppError::Unauthorized("Invalid password".to_string()))?;

    if !valid {
        return Err(AppError::Unauthorized("Invalid password".to_string()));
    }

    // Wyłącz MFA
    sqlx::query!(
        "UPDATE users SET mfa_enabled = false, mfa_secret = NULL WHERE id = $1",
        req.user_id
    )
    .execute(&state.db)
    .await?;

    Ok(Json(DisableMfaResponse {
        success: true,
        message: "MFA disabled successfully".to_string(),
    }))
}

// ==================== TOKEN REFRESH & LOGOUT ====================

#[derive(Debug, serde::Deserialize)]
pub struct RefreshTokenRequest {
    pub refresh_token: String,
}

#[derive(Debug, serde::Serialize)]
pub struct RefreshTokenResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: i64,
}

/// Odświeża access token używając refresh tokena
///
/// # Endpoint
/// POST /auth/refresh
///
/// # Request
/// ```json
/// {
///   "refresh_token": "eyJ..."
/// }
/// ```
///
/// # Response
/// ```json
/// {
///   "access_token": "eyJ...",
///   "refresh_token": "eyJ...",
///   "expires_in": 86400
/// }
/// ```
pub async fn refresh_token(
    State(state): State<AppState>,
    Json(req): Json<RefreshTokenRequest>,
) -> AppResult<Json<RefreshTokenResponse>> {
    // Weryfikuj refresh token
    let claims = crate::auth::jwt::decode_token(&req.refresh_token, &state.config.jwt_secret)
        .map_err(|_| AppError::Unauthorized("Invalid refresh token".to_string()))?;

    // Sprawdź czy użytkownik istnieje
    let user = sqlx::query_as!(
        crate::models::user::User,
        r#"
        SELECT id, email, email_hash, password_hash, first_name, last_name, phone, 
               role as "role: UserRole", mfa_enabled, mfa_secret, email_verified, is_active, created_at, updated_at
        FROM users 
        WHERE id = $1
        "#,
        claims.sub
    )
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

    // Generuj nową parę tokenów
    let token_pair = generate_token_pair(
        user.id,
        user.email.clone(),
        user.role.clone(),
        &state.config,
    )
    .map_err(|e| AppError::Internal(format!("Token generation failed: {}", e)))?;

    Ok(Json(RefreshTokenResponse {
        access_token: token_pair.access_token,
        refresh_token: token_pair.refresh_token,
        expires_in: token_pair.expires_in,
    }))
}

#[derive(Debug, serde::Deserialize)]
pub struct LogoutRequest {
    pub refresh_token: String,
}

#[derive(Debug, serde::Serialize)]
pub struct LogoutResponse {
    pub success: bool,
    pub message: String,
}

/// Wylogowuje użytkownika (unieważnia refresh token)
///
/// # Endpoint
/// POST /auth/logout
///
/// # Request
/// ```json
/// {
///   "refresh_token": "eyJ..."
/// }
/// ```
///
/// # Response
/// ```json
/// {
///   "success": true,
///   "message": "Logged out successfully"
/// }
/// ```
///
/// # Uwaga
/// W pełnej implementacji należy dodać blacklistę tokenów (Redis)
pub async fn logout(
    State(_state): State<AppState>,
    Json(_req): Json<LogoutRequest>,
) -> AppResult<Json<LogoutResponse>> {
    // TODO: Dodanie tokenu do blacklisty (Redis)
    // Na razie tylko zwracamy sukces - klient powinien usunąć tokeny

    Ok(Json(LogoutResponse {
        success: true,
        message: "Logged out successfully".to_string(),
    }))
}
