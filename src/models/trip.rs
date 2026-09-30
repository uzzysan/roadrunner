//! Models for trip execution and stop-event log (Phase 2).

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// Lifecycle status of a trip execution.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, sqlx::Type, Serialize, Deserialize)]
#[sqlx(type_name = "trip_status", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum TripStatus {
    #[default]
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
    Incomplete,
}

/// Type of stop event recorded by the geofence primitive.
#[derive(Debug, Clone, Copy, PartialEq, Eq, sqlx::Type, Serialize, Deserialize)]
#[sqlx(type_name = "trip_event_type", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum TripEventType {
    Arrived,
    Departed,
}

/// Database row for a trip execution.
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct Trip {
    pub id: Uuid,
    pub vehicle_id: Uuid,
    pub route_id: Uuid,
    pub driver_id: Option<Uuid>,
    pub trip_date: NaiveDate,
    pub started_at: Option<DateTime<Utc>>,
    pub ended_at: Option<DateTime<Utc>>,
    pub status: TripStatus,
    pub retain_until: NaiveDate,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Database row for a stop-arrival/departure event.
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct TripEvent {
    pub id: Uuid,
    pub trip_id: Uuid,
    pub stop_id: Uuid,
    pub schedule_id: Option<Uuid>,
    pub event_type: TripEventType,
    pub occurred_at: DateTime<Utc>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub delay_seconds: Option<i32>,
    pub created_at: DateTime<Utc>,
}

// ── Request types ─────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct CreateTripRequest {
    pub vehicle_id: Uuid,
    pub route_id: Uuid,
    pub driver_id: Option<Uuid>,
    pub trip_date: NaiveDate,
    /// Defaults to trip_date + 30 days when omitted.
    pub retain_until: Option<NaiveDate>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateTripRequest {
    pub status: Option<TripStatus>,
    pub started_at: Option<DateTime<Utc>>,
    pub ended_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize)]
pub struct CreateTripEventRequest {
    pub stop_id: Uuid,
    pub schedule_id: Option<Uuid>,
    pub event_type: TripEventType,
    pub occurred_at: DateTime<Utc>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub delay_seconds: Option<i32>,
}

// ── Query parameters ──────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct TripsQuery {
    pub vehicle_id: Option<Uuid>,
    pub route_id: Option<Uuid>,
    pub trip_date: Option<NaiveDate>,
    pub status: Option<TripStatus>,
}

// ── Response types ─────────────────────────────────────────────────────────────

/// Trip row as returned by the API (same fields as Trip for now).
pub type TripResponse = Trip;

/// Trip with its full event log.
#[derive(Debug, Serialize)]
pub struct TripDetailResponse {
    #[serde(flatten)]
    pub trip: Trip,
    pub events: Vec<TripEvent>,
}
