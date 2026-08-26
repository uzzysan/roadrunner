use axum::{http::StatusCode, response::IntoResponse, Json};
use serde_json::json;

/// Centralny typ błędu aplikacji
///
/// Implementuje IntoResponse dla automatycznej konwersji do HTTP response
#[derive(Debug)]
pub enum AppError {
    /// Błąd walidacji danych wejściowych (400)
    BadRequest(String),
    /// Brak autoryzacji (401)
    Unauthorized(String),
    /// Brak uprawnień (403)
    Forbidden(String),
    /// Zasób nie znaleziony (404)
    NotFound(String),
    /// Konflikt danych (409)
    Conflict(String),
    /// Błąd wewnętrzny serwera (500)
    Internal(String),
    /// Błąd bazy danych
    Database(sqlx::Error),
    /// Błąd walidacji (validator crate)
    Validation(validator::ValidationErrors),
    /// Błąd bazy danych ze zserializowaną treścią (transit handlers: routes/stops/schedules)
    DatabaseError(String),
    /// Błąd walidacji ze zserializowaną treścią (transit handlers)
    ValidationError(String),
    /// Błąd wewnętrzny ze zserializowaną treścią (transit handlers)
    InternalError(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        let (status, message) = match self {
            Self::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
            Self::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, msg),
            Self::Forbidden(msg) => (StatusCode::FORBIDDEN, msg),
            Self::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
            Self::Conflict(msg) => (StatusCode::CONFLICT, msg),
            Self::Internal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
            Self::Database(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
            Self::Validation(e) => (StatusCode::BAD_REQUEST, e.to_string()),
            Self::DatabaseError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
            Self::ValidationError(msg) => (StatusCode::BAD_REQUEST, msg),
            Self::InternalError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
        };

        let body = Json(json!({
            "success": false,
            "error": message,
            "status": status.as_u16()
        }));

        (status, body).into_response()
    }
}

impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        match err {
            sqlx::Error::RowNotFound => Self::NotFound("Resource not found".to_string()),
            sqlx::Error::Database(db_err) => {
                if db_err.is_unique_violation() {
                    Self::Conflict("Resource already exists".to_string())
                } else {
                    Self::Database(sqlx::Error::Database(db_err))
                }
            }
            _ => Self::Database(err),
        }
    }
}

impl From<validator::ValidationErrors> for AppError {
    fn from(err: validator::ValidationErrors) -> Self {
        Self::Validation(err)
    }
}

/// Typ wyniku używany w handlerach
pub type AppResult<T> = Result<T, AppError>;
