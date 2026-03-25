pub mod driver;
pub mod gps;
pub mod incident;
pub mod payment;
pub mod route;
pub mod schedule;
pub mod stop;
pub mod student;
pub mod ticket;
pub mod user;
pub mod vehicle;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaseEntity {
    pub id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
