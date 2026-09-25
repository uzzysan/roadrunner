use axum::{
    extract::{Path, State},
    Json,
};
use chrono::{Duration, Utc};
use sha2::{Digest, Sha256};
use sqlx::{Postgres, Transaction};
use uuid::Uuid;

use crate::{
    auth::middleware::AuthUser,
    errors::{AppError, AppResult},
    models::ticket::{
        CreateTicketRequest, Ticket, TicketResponse, TicketStatus, TicketType,
        ValidateTicketRequest, ValidationResponse, ValidationResult,
    },
    models::user::UserRole,
    state::AppState,
    tickets::qr::{generate_ticket_qr, is_valid_ticket_code},
};

/// Tworzy nowy bilet
///
/// # Endpoint
/// POST /tickets
///
/// # Response
/// ```json
/// {
///   "id": "uuid",
///   "ticket_type": "single",
///   "status": "active",
///   "qr_code": "data:image/svg+xml;base64,...",
///   "price": 10.00,
///   "currency": "PLN",
///   "valid_until": "2026-03-27T12:00:00Z"
/// }
/// ```
pub async fn create_ticket(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Json(req): Json<CreateTicketRequest>,
) -> AppResult<Json<TicketResponse>> {
    // Generuj kod i QR
    let (ticket_code, _qr_code) = generate_ticket_qr()?;

    // Określ cenę na podstawie typu biletu
    let (price, currency, validity_days) = match req.ticket_type {
        TicketType::Single => (500, "PLN", 1),     // 5 PLN, 1 dzień
        TicketType::Weekly => (2500, "PLN", 7),    // 25 PLN, 7 dni
        TicketType::Monthly => (8000, "PLN", 30),  // 80 PLN, 30 dni
        TicketType::Discounted => (250, "PLN", 1), // 2.50 PLN, 1 dzień
    };

    let valid_until = Utc::now() + Duration::days(validity_days);

    // Zapisz bilet w bazie
    let ticket = sqlx::query_as!(
        Ticket,
        r#"
        INSERT INTO tickets (
            user_id, ticket_type, status, qr_code, price, currency,
            valid_until, route_id, start_stop_id, end_stop_id
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
        RETURNING 
            id, user_id, ticket_type as "ticket_type: TicketType",
            status as "status: TicketStatus", qr_code, price, currency,
            created_at, valid_until, used_at,
            route_id, start_stop_id, end_stop_id, metadata
        "#,
        user.sub,
        req.ticket_type as TicketType,
        TicketStatus::Active as TicketStatus,
        ticket_code,
        price,
        currency,
        valid_until,
        req.route_id,
        req.start_stop_id,
        req.end_stop_id
    )
    .fetch_one(&state.db)
    .await?;

    Ok(Json(TicketResponse::from(ticket)))
}

/// Pobiera listę biletów użytkownika
///
/// # Endpoint
/// GET /tickets
///
/// # Response
/// ```json
/// [{
///   "id": "uuid",
///   "ticket_type": "single",
///   "status": "active",
///   ...
/// }]
/// ```
pub async fn list_tickets(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
) -> AppResult<Json<Vec<TicketResponse>>> {
    let tickets = sqlx::query_as!(
        Ticket,
        r#"
        SELECT 
            id, user_id, ticket_type as "ticket_type: TicketType",
            status as "status: TicketStatus", qr_code, price, currency,
            created_at, valid_until, used_at,
            route_id, start_stop_id, end_stop_id, metadata
        FROM tickets
        WHERE user_id = $1
        ORDER BY created_at DESC
        "#,
        user.sub
    )
    .fetch_all(&state.db)
    .await?;

    let responses: Vec<TicketResponse> = tickets.into_iter().map(TicketResponse::from).collect();

    Ok(Json(responses))
}

/// Pobiera szczegóły biletu
///
/// # Endpoint
/// GET /tickets/:id
///
/// # Response
/// ```json
/// {
///   "id": "uuid",
///   "ticket_type": "single",
///   "status": "active",
///   ...
/// }
/// ```
pub async fn get_ticket(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Path(ticket_id): Path<Uuid>,
) -> AppResult<Json<TicketResponse>> {
    let ticket = sqlx::query_as!(
        Ticket,
        r#"
        SELECT 
            id, user_id, ticket_type as "ticket_type: TicketType",
            status as "status: TicketStatus", qr_code, price, currency,
            created_at, valid_until, used_at,
            route_id, start_stop_id, end_stop_id, metadata
        FROM tickets
        WHERE id = $1 AND user_id = $2
        "#,
        ticket_id,
        user.sub
    )
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| AppError::NotFound("Ticket not found".to_string()))?;

    Ok(Json(TicketResponse::from(ticket)))
}

/// Waliduje bilet (skanowanie QR)
///
/// # Endpoint
/// POST /tickets/validate
///
/// # Request
/// ```json
/// {
///   "qr_code": "TICKET:uuid:timestamp",
///   "vehicle_id": "uuid",
///   "location": {
///     "latitude": 52.2297,
///     "longitude": 21.0122
///   }
/// }
/// ```
///
/// # Response
/// ```json
/// {
///   "valid": true,
///   "message": "Ticket is valid",
///   "ticket": { ... }
/// }
/// ```
pub async fn validate_ticket(
    State(state): State<AppState>,
    AuthUser(controller): AuthUser,
    Json(req): Json<ValidateTicketRequest>,
) -> AppResult<Json<ValidationResponse>> {
    let mut tx = state.db.begin().await?;
    let qr_fingerprint = hex::encode(Sha256::digest(req.qr_code.as_bytes()));

    if controller.role != UserRole::Attendant {
        record_validation_attempt(
            &mut tx,
            controller.sub,
            None,
            &qr_fingerprint,
            "forbidden",
            &req,
        )
        .await?;
        tx.commit().await?;
        return Err(AppError::Forbidden(
            "Ticket validation requires the attendant role".to_string(),
        ));
    }

    if !is_valid_ticket_code(&req.qr_code) {
        let response = validation_response(ValidationResult::InvalidSignature);
        record_validation_attempt(
            &mut tx,
            controller.sub,
            None,
            &qr_fingerprint,
            response.code.as_str(),
            &req,
        )
        .await?;
        tx.commit().await?;
        return Ok(Json(response));
    }

    // The predicate is re-checked by PostgreSQL after a concurrent row lock is
    // released, so no two scanners can consume the same single-use ticket.
    let consumed = sqlx::query_as::<_, Ticket>(
        r#"
        UPDATE tickets
        SET status = 'used', used_at = NOW()
        WHERE qr_code = $1
          AND status = 'active'
          AND valid_until >= NOW()
          AND ticket_type IN ('single', 'discounted')
        RETURNING id, user_id, ticket_type, status, qr_code, price, currency,
                  created_at, valid_until, used_at,
                  route_id, start_stop_id, end_stop_id, metadata
        "#,
    )
    .bind(&req.qr_code)
    .fetch_optional(&mut *tx)
    .await?;

    if let Some(ticket) = consumed {
        record_validation_attempt(
            &mut tx,
            controller.sub,
            Some(ticket.id),
            &qr_fingerprint,
            ValidationResult::Valid.as_str(),
            &req,
        )
        .await?;
        tx.commit().await?;
        return Ok(Json(validation_response(ValidationResult::Valid)));
    }

    let ticket = sqlx::query_as::<_, Ticket>(
        r#"
        SELECT id, user_id, ticket_type, status, qr_code, price, currency,
               created_at, valid_until, used_at,
               route_id, start_stop_id, end_stop_id, metadata
        FROM tickets
        WHERE qr_code = $1
        FOR UPDATE
        "#,
    )
    .bind(&req.qr_code)
    .fetch_optional(&mut *tx)
    .await?;

    let (result, ticket_id) = match ticket {
        None => (ValidationResult::InvalidSignature, None),
        Some(ticket) if ticket.status == TicketStatus::Used => {
            (ValidationResult::Used, Some(ticket.id))
        }
        Some(ticket) if ticket.valid_until < Utc::now() => {
            sqlx::query(
                "UPDATE tickets SET status = 'expired' WHERE id = $1 AND status = 'active'",
            )
            .bind(ticket.id)
            .execute(&mut *tx)
            .await?;
            (ValidationResult::Expired, Some(ticket.id))
        }
        Some(ticket) if ticket.status == TicketStatus::Expired => {
            (ValidationResult::Expired, Some(ticket.id))
        }
        Some(ticket) if ticket.status == TicketStatus::Active => {
            (ValidationResult::Valid, Some(ticket.id))
        }
        Some(ticket) => (ValidationResult::InvalidSignature, Some(ticket.id)),
    };

    record_validation_attempt(
        &mut tx,
        controller.sub,
        ticket_id,
        &qr_fingerprint,
        result.as_str(),
        &req,
    )
    .await?;
    tx.commit().await?;

    Ok(Json(validation_response(result)))
}

impl ValidationResult {
    fn as_str(self) -> &'static str {
        match self {
            Self::Valid => "valid",
            Self::Used => "used",
            Self::Expired => "expired",
            Self::InvalidSignature => "invalid_signature",
            Self::WrongTenant => "wrong_tenant",
        }
    }
}

fn validation_response(code: ValidationResult) -> ValidationResponse {
    let message = match code {
        ValidationResult::Valid => "Ticket is valid",
        ValidationResult::Used => "Ticket has already been used",
        ValidationResult::Expired => "Ticket has expired",
        ValidationResult::InvalidSignature => "Ticket could not be verified",
        ValidationResult::WrongTenant => "Ticket is not valid for this operator",
    };

    ValidationResponse {
        valid: code == ValidationResult::Valid,
        code,
        message: message.to_string(),
        ticket: None,
    }
}

async fn record_validation_attempt(
    tx: &mut Transaction<'_, Postgres>,
    controller_id: Uuid,
    ticket_id: Option<Uuid>,
    qr_fingerprint: &str,
    result: &str,
    req: &ValidateTicketRequest,
) -> AppResult<()> {
    sqlx::query(
        r#"
        INSERT INTO ticket_validation_attempts
            (controller_id, ticket_id, qr_fingerprint, result, vehicle_id, latitude, longitude)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        "#,
    )
    .bind(controller_id)
    .bind(ticket_id)
    .bind(qr_fingerprint)
    .bind(result)
    .bind(req.vehicle_id)
    .bind(req.location.as_ref().map(|location| location.latitude))
    .bind(req.location.as_ref().map(|location| location.longitude))
    .execute(&mut **tx)
    .await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validation_result_has_stable_codes_and_boolean() {
        let cases = [
            (ValidationResult::Valid, "valid", true),
            (ValidationResult::Used, "used", false),
            (ValidationResult::Expired, "expired", false),
            (
                ValidationResult::InvalidSignature,
                "invalid_signature",
                false,
            ),
            (ValidationResult::WrongTenant, "wrong_tenant", false),
        ];

        for (result, code, valid) in cases {
            let response = validation_response(result);
            assert_eq!(result.as_str(), code);
            assert_eq!(response.valid, valid);
            assert!(response.ticket.is_none());
        }
    }

    #[tokio::test]
    async fn concurrent_single_use_update_succeeds_once_when_test_database_is_available() {
        use sqlx::postgres::PgPoolOptions;
        use std::sync::Arc;
        use tokio::sync::Barrier;

        let Ok(database_url) = std::env::var("TEST_DATABASE_URL") else {
            eprintln!("TEST_DATABASE_URL not set; skipping Postgres ticket concurrency test");
            return;
        };
        let pool = PgPoolOptions::new()
            .max_connections(4)
            .connect(&database_url)
            .await
            .expect("TEST_DATABASE_URL must point to an available disposable Postgres database");
        sqlx::migrate!("./migrations")
            .run(&pool)
            .await
            .expect("test database migrations must succeed");

        let nonce = Uuid::new_v4();
        let email = format!("test_bck21_{nonce}@example.com");
        let email_hash = hex::encode(Sha256::digest(email.as_bytes()));
        let qr_code = format!("TICKET:{nonce}:1700000000");
        let user_id: Uuid = sqlx::query_scalar(
            r#"
            INSERT INTO users (email, email_hash, password_hash, first_name, last_name, role)
            VALUES ($1, $2, 'test-only', 'BCK', 'TwentyOne', 'attendant')
            RETURNING id
            "#,
        )
        .bind(&email)
        .bind(&email_hash)
        .fetch_one(&pool)
        .await
        .expect("test attendant insert must succeed");
        let ticket_id: Uuid = sqlx::query_scalar(
            r#"
            INSERT INTO tickets
                (carrier_id, user_id, ticket_type, status, qr_code, price, currency, valid_until)
            VALUES (
                '00000000-0000-4000-8000-000000000001',
                $1,
                'single',
                'active',
                $2,
                500,
                'PLN',
                NOW() + INTERVAL '1 hour'
            )
            RETURNING id
            "#,
        )
        .bind(user_id)
        .bind(&qr_code)
        .fetch_one(&pool)
        .await
        .expect("test ticket insert must succeed");

        let barrier = Arc::new(Barrier::new(3));
        let mut tasks = Vec::new();
        for _ in 0..2 {
            let pool = pool.clone();
            let barrier = barrier.clone();
            let qr_code = qr_code.clone();
            tasks.push(tokio::spawn(async move {
                barrier.wait().await;
                sqlx::query(
                    r#"
                    UPDATE tickets SET status = 'used', used_at = NOW()
                    WHERE qr_code = $1 AND status = 'active'
                      AND valid_until >= NOW()
                      AND ticket_type IN ('single', 'discounted')
                    "#,
                )
                .bind(qr_code)
                .execute(&pool)
                .await
                .expect("concurrent validation update must succeed")
                .rows_affected()
            }));
        }
        barrier.wait().await;

        let mut consumed = 0;
        for task in tasks {
            consumed += task.await.expect("validation task must not panic");
        }

        sqlx::query("DELETE FROM tickets WHERE id = $1")
            .bind(ticket_id)
            .execute(&pool)
            .await
            .expect("test ticket cleanup must succeed");
        sqlx::query("DELETE FROM users WHERE id = $1")
            .bind(user_id)
            .execute(&pool)
            .await
            .expect("test user cleanup must succeed");

        assert_eq!(consumed, 1);
    }
}
