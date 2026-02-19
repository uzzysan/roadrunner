use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct GpsPosition {
    pub id: Uuid,
    pub vehicle_id: Uuid,
    pub assignment_id: Option<Uuid>,
    pub position: serde_json::Value, // PostGIS point as JSON
    pub speed_kmh: Option<f64>,
    pub heading: Option<i32>,
    pub accuracy_m: Option<f64>,
    pub recorded_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GpsUpdateRequest {
    pub vehicle_id: Uuid,
    pub latitude: f64,
    pub longitude: f64,
    pub speed_kmh: Option<f64>,
    pub heading: Option<i32>,
    pub accuracy_m: Option<f64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct GpsPositionResponse {
    pub vehicle_id: Uuid,
    pub latitude: f64,
    pub longitude: f64,
    pub speed_kmh: Option<f64>,
    pub heading: Option<i32>,
    pub recorded_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub struct VehicleLocation {
    pub vehicle_id: Uuid,
    pub registration_number: String,
    pub route_id: Option<Uuid>,
    pub route_number: Option<String>,
    pub latitude: f64,
    pub longitude: f64,
    pub speed_kmh: Option<f64>,
    pub heading: Option<i32>,
    pub next_stop_name: Option<String>,
    pub next_stop_eta: Option<i32>, // sekundy
    pub recorded_at: DateTime<Utc>,
}
