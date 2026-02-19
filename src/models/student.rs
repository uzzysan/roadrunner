use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "relationship_type")]
pub enum RelationshipType {
    Mother,
    Father,
    LegalGuardian,
    Other,
}

impl Default for RelationshipType {
    fn default() -> Self {
        RelationshipType::Other
    }
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct Student {
    pub id: Uuid,
    pub first_name: String,
    pub last_name: String,
    pub student_id: Option<String>,
    pub school_id: Option<Uuid>,
    pub default_stop_id: Option<Uuid>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct ParentStudentLink {
    pub id: Uuid,
    pub parent_id: Uuid,
    pub student_id: Uuid,
    pub relationship_type: RelationshipType,
    pub is_primary: bool,
    pub can_receive_alerts: bool,
    pub invite_token: Option<String>,
    pub invite_expires_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StudentWithParents {
    #[serde(flatten)]
    pub student: Student,
    pub parents: Vec<ParentInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParentInfo {
    pub id: Uuid,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub relationship: RelationshipType,
    pub is_primary: bool,
}
