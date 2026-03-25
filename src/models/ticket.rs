use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// Typ biletu
#[derive(Debug, Clone, Copy, PartialEq, Eq, sqlx::Type, Serialize, Deserialize)]
#[sqlx(type_name = "ticket_type", rename_all = "snake_case")]
pub enum TicketType {
    /// Bilet jednorazowy
    Single,
    /// Bilet okresowy (miesięczny)
    Monthly,
    /// Bilet okresowy (tygodniowy)
    Weekly,
    /// Bilet ulgowy
    Discounted,
}

/// Status biletu
#[derive(Debug, Clone, Copy, PartialEq, Eq, sqlx::Type, Serialize, Deserialize)]
#[sqlx(type_name = "ticket_status", rename_all = "snake_case")]
pub enum TicketStatus {
    /// Aktywny (ważny)
    Active,
    /// Użyty (skasowany)
    Used,
    /// Wygasły
    Expired,
    /// Anulowany
    Cancelled,
}

/// Bilet
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct Ticket {
    pub id: Uuid,
    pub user_id: Uuid,
    pub ticket_type: TicketType,
    pub status: TicketStatus,
    /// Kod QR (base64 encoded)
    pub qr_code: String,
    /// Cena w groszach (np. 1000 = 10.00 PLN)
    pub price: i32,
    /// Waluta (PLN, EUR, USD)
    pub currency: String,
    /// Data utworzenia
    pub created_at: DateTime<Utc>,
    /// Data ważności
    pub valid_until: DateTime<Utc>,
    /// Data użycia (jeśli użyty)
    pub used_at: Option<DateTime<Utc>>,
    /// ID trasy (opcjonalnie)
    pub route_id: Option<Uuid>,
    /// ID przystanku początkowego
    pub start_stop_id: Option<Uuid>,
    /// ID przystanku końcowego
    pub end_stop_id: Option<Uuid>,
    /// Dodatkowe metadane (JSON)
    pub metadata: Option<serde_json::Value>,
}

/// Request tworzenia biletu
#[derive(Debug, Deserialize)]
pub struct CreateTicketRequest {
    pub ticket_type: TicketType,
    pub route_id: Option<Uuid>,
    pub start_stop_id: Option<Uuid>,
    pub end_stop_id: Option<Uuid>,
}

/// Response z biletem
#[derive(Debug, Serialize)]
pub struct TicketResponse {
    pub id: Uuid,
    pub ticket_type: String,
    pub status: String,
    pub qr_code: String,
    pub price: f64,
    pub currency: String,
    pub created_at: DateTime<Utc>,
    pub valid_until: DateTime<Utc>,
    pub used_at: Option<DateTime<Utc>>,
    pub route_id: Option<Uuid>,
    pub start_stop_id: Option<Uuid>,
    pub end_stop_id: Option<Uuid>,
}

impl From<Ticket> for TicketResponse {
    fn from(ticket: Ticket) -> Self {
        Self {
            id: ticket.id,
            ticket_type: format!("{:?}", ticket.ticket_type).to_lowercase(),
            status: format!("{:?}", ticket.status).to_lowercase(),
            qr_code: ticket.qr_code,
            price: ticket.price as f64 / 100.0,
            currency: ticket.currency,
            created_at: ticket.created_at,
            valid_until: ticket.valid_until,
            used_at: ticket.used_at,
            route_id: ticket.route_id,
            start_stop_id: ticket.start_stop_id,
            end_stop_id: ticket.end_stop_id,
        }
    }
}

/// Request walidacji biletu (skanowanie)
#[derive(Debug, Deserialize)]
pub struct ValidateTicketRequest {
    pub qr_code: String,
    pub vehicle_id: Option<Uuid>,
    pub driver_id: Option<Uuid>,
    pub location: Option<GpsLocation>,
}

#[derive(Debug, Deserialize)]
pub struct GpsLocation {
    pub latitude: f64,
    pub longitude: f64,
}

/// Response z walidacji
#[derive(Debug, Serialize)]
pub struct ValidationResponse {
    pub valid: bool,
    pub message: String,
    pub ticket: Option<TicketResponse>,
}
