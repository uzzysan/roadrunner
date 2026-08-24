use axum::{
    extract::{Path, State},
    Json,
};
use uuid::Uuid;

use crate::{
    auth::middleware::AuthUser,
    config::Config,
    errors::{AppError, AppResult},
    models::payment::{
        CreatePaymentRequest, Payment, PaymentHistoryResponse, PaymentMethod, PaymentResponse,
        PaymentStatus,
    },
    models::ticket::{Ticket, TicketStatus, TicketType},
    payments::stripe::{create_payment_record, update_payment_status, StripeService},
    state::AppState,
    tickets::TicketPricing,
};

/// Tworzy nową płatność (inicjuje płatność w Stripe)
/// 
/// # Endpoint
/// POST /payments
/// 
/// # Request
/// ```json
/// {
///   "ticket_type": "single",
///   "payment_method": "card"
/// }
/// ```
/// 
/// # Response
/// ```json
/// {
///   "id": "uuid",
///   "amount": 5.00,
///   "currency": "PLN",
///   "status": "pending",
///   "client_secret": "pi_..._secret_..."
/// }
/// ```
pub async fn create_payment(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Json(req): Json<CreatePaymentRequest>,
) -> AppResult<Json<PaymentResponse>> {
    // NOTE (2026-08-24 status review): this handler originally computed the price from a
    // `req.ticket_type` field that no longer exists on `CreatePaymentRequest` (the request
    // shape moved to `ticket_id` + `payment_method` — pay for an already-issued ticket —
    // without this function being updated to match, which is why it failed to compile).
    // Fixed by looking the ticket up and pricing/paying against its stored `price`/`currency`.
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
        req.ticket_id,
        user.sub
    )
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| AppError::NotFound("Ticket not found".to_string()))?;

    let amount = ticket.price;
    let currency = ticket.currency.as_str();
    let description = Some(TicketPricing::get_name(ticket.ticket_type));

    // Utwórz rekord płatności w bazie
    let payment = create_payment_record(
        &state.db,
        user.sub,
        Some(ticket.id),
        amount,
        currency,
        description,
    )
    .await?;

    // Utwórz PaymentIntent w Stripe
    let stripe_service = StripeService::new(&state.config);
    let intent = stripe_service
        .create_payment_intent(amount as i64, currency, description)
        .await?;

    // Zaktualizuj rekord z ID PaymentIntent
    let payment = update_payment_status(
        &state.db,
        payment.id,
        PaymentStatus::Pending,
        Some(intent.id.as_str()),
    )
    .await?;

    let mut response = PaymentResponse::from(payment);
    response.client_secret = intent.client_secret;

    Ok(Json(response))
}

/// Pobiera szczegóły płatności
/// 
/// # Endpoint
/// GET /payments/:id
/// 
/// # Response
/// ```json
/// {
///   "id": "uuid",
///   "amount": 5.00,
///   "currency": "PLN",
///   "status": "succeeded",
///   ...
/// }
/// ```
pub async fn get_payment(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Path(payment_id): Path<Uuid>,
) -> AppResult<Json<PaymentResponse>> {
    let payment = sqlx::query_as!(
        Payment,
        r#"
        SELECT 
            id, user_id, ticket_id, stripe_payment_intent_id, stripe_customer_id,
            amount, currency, status as "status: PaymentStatus",
            payment_method as "payment_method: PaymentMethod",
            description, created_at, updated_at, metadata
        FROM payments
        WHERE id = $1 AND user_id = $2
        "#,
        payment_id,
        user.sub
    )
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| AppError::NotFound("Payment not found".to_string()))?;

    Ok(Json(PaymentResponse::from(payment)))
}

/// Pobiera historię płatności użytkownika
/// 
/// # Endpoint
/// GET /payments
/// 
/// # Response
/// ```json
/// [{
///   "id": "uuid",
///   "amount": 5.00,
///   "currency": "PLN",
///   "status": "succeeded",
///   ...
/// }]
/// ```
pub async fn list_payments(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
) -> AppResult<Json<Vec<PaymentHistoryResponse>>> {
    let payments = sqlx::query_as!(
        Payment,
        r#"
        SELECT 
            id, user_id, ticket_id, stripe_payment_intent_id, stripe_customer_id,
            amount, currency, status as "status: PaymentStatus",
            payment_method as "payment_method: PaymentMethod",
            description, created_at, updated_at, metadata
        FROM payments
        WHERE user_id = $1
        ORDER BY created_at DESC
        "#,
        user.sub
    )
    .fetch_all(&state.db)
    .await?;

    let responses: Vec<PaymentHistoryResponse> = payments
        .into_iter()
        .map(PaymentHistoryResponse::from)
        .collect();

    Ok(Json(responses))
}

/// Webhook Stripe - obsługa zdarzeń płatności
/// 
/// # Endpoint
/// POST /webhooks/stripe
/// 
/// # Request (from Stripe)
/// Stripe webhook payload
/// 
/// # Response
/// ```json
/// { "received": true }
/// ```
pub async fn stripe_webhook(
    State(state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> AppResult<Json<serde_json::Value>> {
    // Pobierz typ zdarzenia
    let event_type = payload
        .get("type")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::BadRequest("Missing event type".to_string()))?;

    let data = payload
        .get("data")
        .and_then(|d| d.get("object"))
        .ok_or_else(|| AppError::BadRequest("Missing event data".to_string()))?;

    match event_type {
        "payment_intent.succeeded" => {
            let intent_id = data
                .get("id")
                .and_then(|v| v.as_str())
                .ok_or_else(|| AppError::BadRequest("Missing intent ID".to_string()))?;

            // Znajdź płatność po ID PaymentIntent
            let payment = sqlx::query_as!(
                Payment,
                r#"
                SELECT 
                    id, user_id, ticket_id, stripe_payment_intent_id, stripe_customer_id,
                    amount, currency, status as "status: PaymentStatus",
                    payment_method as "payment_method: PaymentMethod",
                    description, created_at, updated_at, metadata
                FROM payments
                WHERE stripe_payment_intent_id = $1
                "#,
                intent_id
            )
            .fetch_optional(&state.db)
            .await?;

            if let Some(payment) = payment {
                // Zaktualizuj status na succeeded
                update_payment_status(
                    &state.db,
                    payment.id,
                    PaymentStatus::Succeeded,
                    None,
                )
                .await?;

                // TODO: Utwórz bilet po udanej płatności
            }
        }
        "payment_intent.payment_failed" => {
            let intent_id = data
                .get("id")
                .and_then(|v| v.as_str())
                .ok_or_else(|| AppError::BadRequest("Missing intent ID".to_string()))?;

            let payment = sqlx::query_as!(
                Payment,
                r#"
                SELECT 
                    id, user_id, ticket_id, stripe_payment_intent_id, stripe_customer_id,
                    amount, currency, status as "status: PaymentStatus",
                    payment_method as "payment_method: PaymentMethod",
                    description, created_at, updated_at, metadata
                FROM payments
                WHERE stripe_payment_intent_id = $1
                "#,
                intent_id
            )
            .fetch_optional(&state.db)
            .await?;

            if let Some(payment) = payment {
                update_payment_status(
                    &state.db,
                    payment.id,
                    PaymentStatus::Failed,
                    None,
                )
                .await?;
            }
        }
        _ => {
            // Ignoruj inne zdarzenia
        }
    }

    Ok(Json(serde_json::json!({ "received": true })))
}
