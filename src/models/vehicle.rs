use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct Vehicle {
    pub id: Uuid,
    pub registration_number: String,
    pub name: Option<String>,
    pub vehicle_type: String,
    pub capacity: Option<i32>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct VehicleAssignment {
    pub id: Uuid,
    pub vehicle_id: Uuid,
    pub route_id: Uuid,
    pub driver_id: Option<Uuid>,
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
    pub direction: String,
    pub is_active: bool,
    // Join fields
    pub vehicle_registration: Option<String>,
    pub route_number: Option<String>,
    pub route_name: Option<String>,
    pub driver_name: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateVehicleRequest {
    pub registration_number: String,
    pub name: Option<String>,
    pub vehicle_type: Option<String>,
    pub capacity: Option<i32>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateAssignmentRequest {
    pub vehicle_id: Uuid,
    pub route_id: Uuid,
    pub driver_id: Option<Uuid>,
    pub direction: Option<String>,
}
