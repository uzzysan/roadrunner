use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(rename = "route_type", rename_all = "lowercase")]
pub enum RouteType {
    Regular,
    School,
    Night,
}

impl Default for RouteType {
    fn default() -> Self {
        RouteType::Regular
    }
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct Route {
    pub id: Uuid,
    pub name: String,
    pub number: String,
    pub description: Option<String>,
    pub route_type: RouteType,
    pub color: String,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct RouteStop {
    pub id: Uuid,
    pub route_id: Uuid,
    pub stop_id: Uuid,
    pub sequence: i32,
    pub scheduled_duration_from_start: Option<i32>,
    pub is_active: bool,
    // Join fields
    pub stop_name: Option<String>,
    pub stop_location: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateRouteRequest {
    pub name: String,
    pub number: String,
    pub description: Option<String>,
    pub route_type: Option<RouteType>,
    pub color: Option<String>,
    pub stops: Vec<RouteStopInput>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RouteStopInput {
    pub stop_id: Uuid,
    pub sequence: i32,
    pub scheduled_duration_from_start: Option<i32>,
}
