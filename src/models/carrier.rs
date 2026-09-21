use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// A transport operator that owns tenant-scoped RoadRunner data.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Carrier {
    pub id: Uuid,
    pub slug: String,
    pub name: String,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Links a global user identity to a carrier without making authentication
/// records tenant-scoped.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct CarrierMembership {
    pub carrier_id: Uuid,
    pub user_id: Uuid,
    pub created_at: DateTime<Utc>,
}
