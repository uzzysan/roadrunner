use axum::{
    body::Bytes,
    extract::{Path, State},
    http::HeaderMap,
    Json,
};
use hmac::{Hmac, Mac};
use sha2::Sha256;
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

use crate::{
    auth::middleware::AuthUser,
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
    headers: HeaderMap,
    body: Bytes,
) -> AppResult<Json<serde_json::Value>> {
    let secret = state
        .config
        .stripe_webhook_secret
        .as_deref()
        .ok_or_else(|| AppError::Internal("Stripe webhook secret is not configured".into()))?;
    let signature = headers
        .get("stripe-signature")
        .and_then(|value| value.to_str().ok())
        .ok_or_else(|| AppError::BadRequest("Missing Stripe signature".into()))?;
    verify_stripe_signature(&body, signature, secret, unix_time())?;

    let payload: serde_json::Value = serde_json::from_slice(&body)
        .map_err(|_| AppError::BadRequest("Invalid webhook JSON".into()))?;
    let event_id = required_str(&payload, "id")?;
    let event_type = required_str(&payload, "type")?;
    if payload.get("object").and_then(|v| v.as_str()) != Some("event") {
        return Err(AppError::BadRequest("Invalid Stripe event".into()));
    }
    let api_version = payload.get("api_version").and_then(|v| v.as_str());
    let supported = matches!(
        event_type,
        "payment_intent.succeeded" | "payment_intent.payment_failed"
    );
    if supported
        && (api_version.is_none()
            || api_version != state.config.stripe_webhook_api_version.as_deref())
    {
        // Never acknowledge an event whose object shape is not configured for this endpoint.
        return Err(AppError::Internal(
            "Unexpected Stripe event API version".into(),
        ));
    }
    let intent_id = if supported {
        let object = payload
            .get("data")
            .and_then(|v| v.get("object"))
            .ok_or_else(|| AppError::BadRequest("Missing event data".into()))?;
        if object.get("object").and_then(|v| v.as_str()) != Some("payment_intent") {
            return Err(AppError::BadRequest("Invalid PaymentIntent event".into()));
        }
        Some(required_str(object, "id")?)
    } else {
        None
    };

    let mut tx = state.db.begin().await?;
    // The unique index serializes concurrent deliveries: ON CONFLICT waits for the
    // first transaction, then reports a duplicate only after it has committed.
    let inserted = sqlx::query(
        "INSERT INTO stripe_webhook_events (event_id, event_type, stripe_payment_intent_id, api_version) \
         VALUES ($1, $2, $3, $4) ON CONFLICT (event_id) DO NOTHING")
        .bind(event_id).bind(event_type).bind(intent_id).bind(api_version)
        .execute(&mut *tx).await?.rows_affected() == 1;
    if !inserted {
        tx.commit().await?;
        return Ok(Json(serde_json::json!({ "received": true })));
    }

    if let Some(intent_id) = intent_id {
        let result = match event_type {
            "payment_intent.succeeded" => {
                sqlx::query(
                    "UPDATE payments SET status = $1, updated_at = now() \
                 WHERE stripe_payment_intent_id = $2 AND status IN ('pending', 'failed')",
                )
                .bind(PaymentStatus::Succeeded)
                .bind(intent_id)
                .execute(&mut *tx)
                .await?
            }
            "payment_intent.payment_failed" => {
                sqlx::query(
                    "UPDATE payments SET status = $1, updated_at = now() \
                 WHERE stripe_payment_intent_id = $2 AND status = 'pending'",
                )
                .bind(PaymentStatus::Failed)
                .bind(intent_id)
                .execute(&mut *tx)
                .await?
            }
            _ => unreachable!(),
        };
        if result.rows_affected() == 0 {
            let exists: bool = sqlx::query_scalar(
                "SELECT EXISTS(SELECT 1 FROM payments WHERE stripe_payment_intent_id = $1)",
            )
            .bind(intent_id)
            .fetch_one(&mut *tx)
            .await?;
            if !exists {
                // Rollback the event marker, allowing Stripe to retry after the payment is saved.
                return Err(AppError::Internal("PaymentIntent not yet recorded".into()));
            }
        }
    }
    tx.commit().await?;
    Ok(Json(serde_json::json!({ "received": true })))
}

fn required_str<'a>(value: &'a serde_json::Value, field: &str) -> AppResult<&'a str> {
    value
        .get(field)
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .ok_or_else(|| AppError::BadRequest(format!("Missing {field}")))
}

fn unix_time() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

fn verify_stripe_signature(body: &[u8], header: &str, secret: &str, now: i64) -> AppResult<()> {
    if header.len() > 4096 || secret.is_empty() {
        return Err(AppError::BadRequest("Invalid Stripe signature".into()));
    }
    let mut timestamp = None;
    let mut signatures = Vec::new();
    for part in header.split(',') {
        let (key, value) = part
            .trim()
            .split_once('=')
            .ok_or_else(|| AppError::BadRequest("Invalid Stripe signature".into()))?;
        let value = value.trim();
        match key.trim() {
            "t" => {
                if timestamp.is_some() {
                    return Err(AppError::BadRequest("Invalid Stripe signature".into()));
                }
                timestamp = Some(
                    value
                        .parse::<i64>()
                        .map_err(|_| AppError::BadRequest("Invalid Stripe signature".into()))?,
                );
            }
            "v1" if !value.is_empty() => signatures.push(value),
            "v1" => return Err(AppError::BadRequest("Invalid Stripe signature".into())),
            _ => (),
        }
    }
    let timestamp =
        timestamp.ok_or_else(|| AppError::BadRequest("Invalid Stripe signature".into()))?;
    if (now as i128 - timestamp as i128).abs() > 300 {
        return Err(AppError::BadRequest("Expired Stripe signature".into()));
    }
    if !signatures.iter().any(|signature| {
        hex::decode(signature).ok().is_some_and(|decoded| {
            Hmac::<Sha256>::new_from_slice(secret.as_bytes()).is_ok_and(|mut check| {
                check.update(timestamp.to_string().as_bytes());
                check.update(b".");
                check.update(body);
                check.verify_slice(&decoded).is_ok()
            })
        })
    }) {
        return Err(AppError::BadRequest("Invalid Stripe signature".into()));
    }
    Ok(())
}

#[cfg(test)]
mod webhook_tests {
    use super::*;

    fn signed(body: &[u8], secret: &str, timestamp: i64) -> String {
        let mut mac = Hmac::<Sha256>::new_from_slice(secret.as_bytes()).unwrap();
        mac.update(timestamp.to_string().as_bytes());
        mac.update(b".");
        mac.update(body);
        format!(
            "t={timestamp},v1={}",
            hex::encode(mac.finalize().into_bytes())
        )
    }

    #[test]
    fn accepts_exact_raw_body_and_rotated_signature() {
        let body = br#"{ "id": "evt_1", "object": "event" }"#;
        let header = format!(
            "t=1700000000,v1={},v1={}",
            "00".repeat(32),
            signed(body, "whsec_test", 1700000000)
                .split("v1=")
                .nth(1)
                .unwrap()
        );
        assert!(verify_stripe_signature(body, &header, "whsec_test", 1700000000).is_ok());
        assert!(verify_stripe_signature(
            br#"{"id":"evt_1","object":"event"}"#,
            &header,
            "whsec_test",
            1700000000
        )
        .is_err());
    }

    #[test]
    fn rejects_missing_invalid_and_replayed_signatures() {
        let body = b"{}";
        let header = signed(body, "whsec_test", 1700000000);
        assert!(verify_stripe_signature(body, "", "whsec_test", 1700000000).is_err());
        assert!(verify_stripe_signature(body, &header, "wrong_secret", 1700000000).is_err());
        assert!(verify_stripe_signature(body, &header, "whsec_test", 1700000301).is_err());
        assert!(verify_stripe_signature(body, &header, "whsec_test", 1699999699).is_err());
    }

    #[test]
    fn rejects_malformed_or_ambiguous_signature_headers() {
        let body = b"{}";
        let valid = signed(body, "whsec_test", 1700000000);
        let digest = valid.split("v1=").nth(1).unwrap();

        for header in [
            "garbage",
            "t=not-a-number,v1=00",
            "t=1700000000,v1=",
            &format!("t=1700000000,t=1700000000,v1={digest}"),
        ] {
            assert!(verify_stripe_signature(body, header, "whsec_test", 1700000000).is_err());
        }
    }

    /// Uses a disposable database configured through TEST_DATABASE_URL. When the variable is
    /// absent, this test is intentionally skipped so the default unit-test suite stays hermetic.
    #[tokio::test]
    async fn concurrent_replay_is_recorded_once_when_test_database_is_available() {
        use sqlx::postgres::PgPoolOptions;
        use std::sync::Arc;
        use tokio::sync::Barrier;

        let Ok(database_url) = std::env::var("TEST_DATABASE_URL") else {
            eprintln!("TEST_DATABASE_URL not set; skipping Postgres concurrency test");
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

        let event_id = format!("evt_test_{}", Uuid::new_v4());
        let barrier = Arc::new(Barrier::new(3));
        let mut tasks = Vec::new();
        for _ in 0..2 {
            let pool = pool.clone();
            let barrier = barrier.clone();
            let event_id = event_id.clone();
            tasks.push(tokio::spawn(async move {
                barrier.wait().await;
                sqlx::query(
                    "INSERT INTO stripe_webhook_events \
                     (event_id, event_type, stripe_payment_intent_id, api_version) \
                     VALUES ($1, 'test.event', NULL, 'test') \
                     ON CONFLICT (event_id) DO NOTHING",
                )
                .bind(event_id)
                .execute(&pool)
                .await
                .expect("concurrent event insert must succeed")
                .rows_affected()
            }));
        }
        barrier.wait().await;

        let mut inserted = 0;
        for task in tasks {
            inserted += task.await.expect("insert task must not panic");
        }
        let stored: i64 =
            sqlx::query_scalar("SELECT count(*) FROM stripe_webhook_events WHERE event_id = $1")
                .bind(&event_id)
                .fetch_one(&pool)
                .await
                .expect("stored event must be queryable");
        sqlx::query("DELETE FROM stripe_webhook_events WHERE event_id = $1")
            .bind(&event_id)
            .execute(&pool)
            .await
            .expect("test event cleanup must succeed");

        assert_eq!(inserted, 1);
        assert_eq!(stored, 1);
    }
}
