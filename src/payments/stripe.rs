use crate::{
    config::Config,
    errors::{AppError, AppResult},
    models::payment::{Payment, PaymentMethod, PaymentStatus},
};
use sqlx::PgPool;
use stripe::{Client, CreatePaymentIntent, Currency, PaymentIntent};
use uuid::Uuid;

/// Serwis do obsługi płatności Stripe
pub struct StripeService {
    client: Client,
}

impl StripeService {
    /// Tworzy nowy serwis Stripe
    pub fn new(config: &Config) -> Self {
        let client = Client::new(config.stripe_secret_key.clone());
        Self { client }
    }

    /// Tworzy PaymentIntent w Stripe
    ///
    /// # Arguments
    /// * `amount` - Kwota w groszach
    /// * `currency` - Waluta (PLN, EUR, USD)
    /// * `description` - Opis płatności
    ///
    /// # Returns
    /// * `PaymentIntent` - Utworzony PaymentIntent
    pub async fn create_payment_intent(
        &self,
        amount: i64,
        currency: &str,
        description: Option<&str>,
    ) -> AppResult<PaymentIntent> {
        let currency = match currency {
            "PLN" => Currency::PLN,
            "EUR" => Currency::EUR,
            "USD" => Currency::USD,
            _ => Currency::PLN,
        };

        let mut create_intent = CreatePaymentIntent::new(amount, currency);

        if let Some(desc) = description {
            create_intent.description = Some(desc);
        }

        // Automatyczne potwierdzenie (dla kart)
        create_intent.automatic_payment_methods =
            Some(stripe::CreatePaymentIntentAutomaticPaymentMethods {
                enabled: true,
                ..Default::default()
            });

        let intent = PaymentIntent::create(&self.client, create_intent)
            .await
            .map_err(|e| AppError::Internal(format!("Stripe error: {}", e)))?;

        Ok(intent)
    }

    /// Pobiera PaymentIntent z Stripe
    ///
    /// # Arguments
    /// * `payment_intent_id` - ID PaymentIntent
    ///
    /// # Returns
    /// * `PaymentIntent` - Pobrany PaymentIntent
    pub async fn retrieve_payment_intent(
        &self,
        payment_intent_id: &str,
    ) -> AppResult<PaymentIntent> {
        let intent = PaymentIntent::retrieve(
            &self.client,
            &payment_intent_id
                .parse::<stripe::PaymentIntentId>()
                .map_err(|_| AppError::BadRequest("Invalid payment intent ID".to_string()))?,
            &[],
        )
        .await
        .map_err(|e| AppError::Internal(format!("Stripe error: {}", e)))?;

        Ok(intent)
    }

    /// Anuluje PaymentIntent
    ///
    /// # Arguments
    /// * `payment_intent_id` - ID PaymentIntent
    ///
    /// # Returns
    /// * `PaymentIntent` - Anulowany PaymentIntent
    pub async fn cancel_payment_intent(&self, payment_intent_id: &str) -> AppResult<PaymentIntent> {
        let intent = PaymentIntent::cancel(
            &self.client,
            &payment_intent_id
                .parse::<stripe::PaymentIntentId>()
                .map_err(|_| AppError::BadRequest("Invalid payment intent ID".to_string()))?,
            stripe::CancelPaymentIntent::default(),
        )
        .await
        .map_err(|e| AppError::Internal(format!("Stripe error: {}", e)))?;

        Ok(intent)
    }
}

/// Tworzy rekord płatności w bazie danych
///
/// # Arguments
/// * `pool` - PgPool
/// * `user_id` - ID użytkownika
/// * `ticket_id` - ID biletu (opcjonalnie)
/// * `amount` - Kwota w groszach
/// * `currency` - Waluta
/// * `description` - Opis
///
/// # Returns
/// * `Payment` - Utworzona płatność
pub async fn create_payment_record(
    pool: &PgPool,
    user_id: Uuid,
    ticket_id: Option<Uuid>,
    amount: i32,
    currency: &str,
    description: Option<&str>,
) -> AppResult<Payment> {
    let payment = sqlx::query_as!(
        Payment,
        r#"
        INSERT INTO payments (user_id, ticket_id, amount, currency, status, description)
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING 
            id, user_id, ticket_id, stripe_payment_intent_id, stripe_customer_id,
            amount, currency, status as "status: PaymentStatus",
            payment_method as "payment_method: PaymentMethod",
            description, created_at, updated_at, metadata
        "#,
        user_id,
        ticket_id,
        amount,
        currency,
        PaymentStatus::Pending as PaymentStatus,
        description
    )
    .fetch_one(pool)
    .await?;

    Ok(payment)
}

/// Aktualizuje status płatności
///
/// # Arguments
/// * `pool` - PgPool
/// * `payment_id` - ID płatności
/// * `status` - Nowy status
/// * `stripe_payment_intent_id` - ID PaymentIntent w Stripe (opcjonalnie)
///
/// # Returns
/// * `Payment` - Zaktualizowana płatność
pub async fn update_payment_status(
    pool: &PgPool,
    payment_id: Uuid,
    status: PaymentStatus,
    stripe_payment_intent_id: Option<&str>,
) -> AppResult<Payment> {
    let payment = sqlx::query_as!(
        Payment,
        r#"
        UPDATE payments 
        SET status = $2, stripe_payment_intent_id = COALESCE($3, stripe_payment_intent_id)
        WHERE id = $1
        RETURNING 
            id, user_id, ticket_id, stripe_payment_intent_id, stripe_customer_id,
            amount, currency, status as "status: PaymentStatus",
            payment_method as "payment_method: PaymentMethod",
            description, created_at, updated_at, metadata
        "#,
        payment_id,
        status as PaymentStatus,
        stripe_payment_intent_id
    )
    .fetch_one(pool)
    .await?;

    Ok(payment)
}
