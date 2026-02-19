use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct Stop {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    // location jako JSON w modelu, PostGIS w bazie
    pub location: serde_json::Value,
    pub geofence_radius_m: i32,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateStopRequest {
    pub name: String,
    pub description: Option<String>,
    pub latitude: f64,
    pub longitude: f64,
    pub geofence_radius_m: Option<i32>,
}

#[derive(Debug, Clone, Serialize)]
pub struct StopResponse {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub latitude: f64,
    pub longitude: f64,
    pub geofence_radius_m: i32,
}
