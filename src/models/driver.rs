//! Model kierowcy (Driver) dla zarządzania flotą
//!
//! Przechowuje informacje o kierowcach, ich uprawnieniach,
//! statusie i przypisaniu do pojazdów.

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// Status kierowcy
#[derive(Debug, Clone, Copy, PartialEq, Eq, sqlx::Type, Serialize, Deserialize)]
#[sqlx(type_name = "driver_status", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum DriverStatus {
    Active,    // Aktywny
    OnLeave,   // Na urlopie
    Suspended, // Zawieszony
    Inactive,  // Nieaktywny
}

impl DriverStatus {
    pub fn to_polish(&self) -> &'static str {
        match self {
            DriverStatus::Active => "Aktywny",
            DriverStatus::OnLeave => "Na urlopie",
            DriverStatus::Suspended => "Zawieszony",
            DriverStatus::Inactive => "Nieaktywny",
        }
    }

    pub fn can_drive(&self) -> bool {
        matches!(self, DriverStatus::Active)
    }
}

/// Model kierowcy w bazie danych
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct Driver {
    pub id: Uuid,
    /// Powiązanie z tabelą users
    pub user_id: Uuid,
    /// Numer pracownika
    pub employee_id: Option<String>,
    /// Numer prawa jazdy
    pub license_number: String,
    /// Kategorie prawa jazdy (np. ["D", "DE"])
    pub license_categories: Vec<String>,
    /// Data ważności prawa jazdy
    pub license_expiry: NaiveDate,
    /// Telefon kontaktowy
    pub phone: String,
    /// Kontakt awaryjny
    pub emergency_contact: Option<String>,
    /// Status kierowcy
    pub status: DriverStatus,
    /// ID przypisanego pojazdu
    pub assigned_vehicle_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Response API dla kierowcy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriverResponse {
    pub id: Uuid,
    pub user_id: Uuid,
    pub employee_id: Option<String>,
    pub license_number: String,
    pub license_categories: Vec<String>,
    pub license_expiry: NaiveDate,
    pub phone: String,
    pub emergency_contact: Option<String>,
    pub status: DriverStatus,
    pub status_name_pl: String,
    pub can_drive: bool,
    pub assigned_vehicle_id: Option<Uuid>,
    pub assigned_vehicle_info: Option<AssignedVehicleInfo>,
    pub created_at: DateTime<Utc>,
}

impl From<Driver> for DriverResponse {
    fn from(d: Driver) -> Self {
        Self {
            id: d.id,
            user_id: d.user_id,
            employee_id: d.employee_id,
            license_number: d.license_number,
            license_categories: d.license_categories,
            license_expiry: d.license_expiry,
            phone: d.phone,
            emergency_contact: d.emergency_contact,
            status: d.status,
            status_name_pl: d.status.to_polish().to_string(),
            can_drive: d.status.can_drive(),
            assigned_vehicle_id: d.assigned_vehicle_id,
            assigned_vehicle_info: None, // Wypełniane w handlerze
            created_at: d.created_at,
        }
    }
}

/// Informacje o przypisanym pojeździe
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssignedVehicleInfo {
    pub id: Uuid,
    pub registration_number: String,
    pub brand: String,
    pub model: String,
}

/// Kierowca z danymi użytkownika
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriverWithUser {
    #[serde(flatten)]
    pub driver: DriverResponse,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub full_name: String,
}

/// Request do tworzenia kierowcy
#[derive(Debug, Deserialize)]
pub struct CreateDriverRequest {
    pub user_id: Uuid,
    pub employee_id: Option<String>,
    pub license_number: String,
    pub license_categories: Vec<String>,
    pub license_expiry: NaiveDate,
    pub phone: String,
    pub emergency_contact: Option<String>,
}

/// Request do aktualizacji kierowcy
#[derive(Debug, Deserialize)]
pub struct UpdateDriverRequest {
    pub employee_id: Option<String>,
    pub license_number: Option<String>,
    pub license_categories: Option<Vec<String>>,
    pub license_expiry: Option<NaiveDate>,
    pub phone: Option<String>,
    pub emergency_contact: Option<String>,
    pub status: Option<DriverStatus>,
}

/// Request do przypisania pojazdu
#[derive(Debug, Deserialize)]
pub struct AssignVehicleRequest {
    pub vehicle_id: Uuid,
}

/// Informacje o kierowcy dla ekranu kierowcy (mobile)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriverDashboardInfo {
    pub driver_id: Uuid,
    pub full_name: String,
    pub phone: String,
    pub assigned_vehicle: Option<DriverVehicleInfo>,
    pub current_route: Option<DriverRouteInfo>,
    pub today_shifts: Vec<DriverShift>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriverVehicleInfo {
    pub id: Uuid,
    pub registration_number: String,
    pub brand: String,
    pub model: String,
    pub capacity: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriverRouteInfo {
    pub id: Uuid,
    pub name: String,
    pub number: String,
    pub color: String,
    pub first_stop: String,
    pub last_stop: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriverShift {
    pub shift_id: Uuid,
    pub route_id: Uuid,
    pub route_name: String,
    pub start_time: String,
    pub end_time: String,
    pub status: String, // scheduled, in_progress, completed
}

/// Sprawdza czy kierowca ma ważne prawo jazdy
pub fn is_license_valid(license_expiry: NaiveDate) -> bool {
    let today = Utc::now().date_naive();
    license_expiry >= today
}

/// Formatuje kategorie prawa jazdy jako string
pub fn format_license_categories(categories: &[String]) -> String {
    categories.join(", ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    #[test]
    fn test_driver_status_can_drive() {
        assert!(DriverStatus::Active.can_drive());
        assert!(!DriverStatus::OnLeave.can_drive());
        assert!(!DriverStatus::Suspended.can_drive());
        assert!(!DriverStatus::Inactive.can_drive());
    }

    #[test]
    fn test_is_license_valid() {
        let future = Utc::now().date_naive() + Duration::days(30);
        let past = Utc::now().date_naive() - Duration::days(30);
        let today = Utc::now().date_naive();

        assert!(is_license_valid(future));
        assert!(!is_license_valid(past));
        assert!(is_license_valid(today));
    }

    #[test]
    fn test_format_license_categories() {
        let cats = vec!["D".to_string(), "DE".to_string(), "C".to_string()];
        assert_eq!(format_license_categories(&cats), "D, DE, C");
    }
}
