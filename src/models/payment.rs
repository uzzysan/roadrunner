use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// Status płatności
#[derive(Debug, Clone, Copy, PartialEq, Eq, sqlx::Type, Serialize, Deserialize)]
#[sqlx(type_name = "payment_status", rename_all = "snake_case")]
pub enum PaymentStatus {
    /// Oczekująca
    Pending,
    /// Zakończona sukcesem
    Succeeded,
    /// Nieudana
    Failed,
    /// Zwrócona
    Refunded,
    /// Anulowana
    Cancelled,
}

/// Metoda płatności
#[derive(Debug, Clone, Copy, PartialEq, Eq, sqlx::Type, Serialize, Deserialize)]
#[sqlx(type_name = "payment_method", rename_all = "snake_case")]
pub enum PaymentMethod {
    /// Karta kredytowa/debetowa
    Card,
    /// BLIK
    Blik,
    /// Przelew bankowy
    BankTransfer,
    /// Apple Pay
    ApplePay,
    /// Google Pay
    GooglePay,
}

/// Płatność
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct Payment {
    pub id: Uuid,
    pub user_id: Uuid,
    pub ticket_id: Option<Uuid>,
    pub stripe_payment_intent_id: Option<String>,
    pub stripe_customer_id: Option<String>,
    pub amount: i32, // w groszach
    pub currency: String,
    pub status: PaymentStatus,
    pub payment_method: Option<PaymentMethod>,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub metadata: Option<serde_json::Value>,
}

/// Request tworzenia płatności
#[derive(Debug, Deserialize)]
pub struct CreatePaymentRequest {
    pub ticket_id: Uuid,
    pub payment_method: PaymentMethod,
}

/// Response z płatnością
#[derive(Debug, Serialize)]
pub struct PaymentResponse {
    pub id: Uuid,
    pub amount: f64,
    pub currency: String,
    pub status: String,
    pub client_secret: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// Request potwierdzenia płatności (webhook)
#[derive(Debug, Deserialize)]
pub struct PaymentWebhookRequest {
    pub id: String,
    pub object: String,
    pub type_: String,
    pub data: serde_json::Value,
}

/// Response z historii płatności
#[derive(Debug, Serialize)]
pub struct PaymentHistoryResponse {
    pub id: Uuid,
    pub amount: f64,
    pub currency: String,
    pub status: String,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
}

impl From<Payment> for PaymentResponse {
    fn from(payment: Payment) -> Self {
        Self {
            id: payment.id,
            amount: payment.amount as f64 / 100.0,
            currency: payment.currency,
            status: format!("{:?}", payment.status).to_lowercase(),
            client_secret: None, // Wypełniane osobno
            created_at: payment.created_at,
        }
    }
}

impl From<Payment> for PaymentHistoryResponse {
    fn from(payment: Payment) -> Self {
        Self {
            id: payment.id,
            amount: payment.amount as f64 / 100.0,
            currency: payment.currency,
            status: format!("{:?}", payment.status).to_lowercase(),
            description: payment.description,
            created_at: payment.created_at,
        }
    }
}
