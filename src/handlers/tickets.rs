use axum::{
    extract::{Path, State},
    Json,
};
use chrono::{Duration, Utc};
use uuid::Uuid;

use crate::{
    auth::middleware::AuthUser,
    errors::{AppError, AppResult},
    models::ticket::{
        CreateTicketRequest, Ticket, TicketResponse, TicketStatus, TicketType,
        ValidateTicketRequest, ValidationResponse,
    },
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
    Json(req): Json<ValidateTicketRequest>,
) -> AppResult<Json<ValidationResponse>> {
    // Sprawdź format kodu
    if !is_valid_ticket_code(&req.qr_code) {
        return Ok(Json(ValidationResponse {
            valid: false,
            message: "Invalid ticket code format".to_string(),
            ticket: None,
        }));
    }

    // Znajdź bilet po kodzie QR
    let ticket = sqlx::query_as!(
        Ticket,
        r#"
        SELECT 
            id, user_id, ticket_type as "ticket_type: TicketType",
            status as "status: TicketStatus", qr_code, price, currency,
            created_at, valid_until, used_at,
            route_id, start_stop_id, end_stop_id, metadata
        FROM tickets
        WHERE qr_code = $1
        "#,
        req.qr_code
    )
    .fetch_optional(&state.db)
    .await?;

    let ticket = match ticket {
        Some(t) => t,
        None => {
            return Ok(Json(ValidationResponse {
                valid: false,
                message: "Ticket not found".to_string(),
                ticket: None,
            }));
        }
    };

    // Sprawdź czy bilet jest aktywny
    if ticket.status != TicketStatus::Active {
        return Ok(Json(ValidationResponse {
            valid: false,
            message: format!("Ticket is {:?}", ticket.status),
            ticket: Some(TicketResponse::from(ticket)),
        }));
    }

    // Sprawdź czy bilet nie wygasł
    if ticket.valid_until < Utc::now() {
        // Oznacz jako wygasły
        sqlx::query!(
            "UPDATE tickets SET status = 'expired' WHERE id = $1",
            ticket.id
        )
        .execute(&state.db)
        .await?;

        return Ok(Json(ValidationResponse {
            valid: false,
            message: "Ticket has expired".to_string(),
            ticket: Some(TicketResponse::from(ticket)),
        }));
    }

    // Oznacz bilet jako użyty
    sqlx::query!(
        "UPDATE tickets SET status = 'used', used_at = NOW() WHERE id = $1",
        ticket.id
    )
    .execute(&state.db)
    .await?;

    // Zapisz walidację w historii
    sqlx::query!(
        r#"
        INSERT INTO ticket_validations 
        (ticket_id, vehicle_id, latitude, longitude, is_valid)
        VALUES ($1, $2, $3, $4, true)
        "#,
        ticket.id,
        req.vehicle_id,
        req.location.as_ref().map(|l| l.latitude),
        req.location.as_ref().map(|l| l.longitude)
    )
    .execute(&state.db)
    .await?;

    Ok(Json(ValidationResponse {
        valid: true,
        message: "Ticket is valid".to_string(),
        ticket: Some(TicketResponse::from(ticket)),
    }))
}
