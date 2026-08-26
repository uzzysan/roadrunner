//! Model incydentu/zdarzenia (Incident) dla systemu awarii
//!
//! Przechowuje informacje o awariach, opóźnieniach i innych
//! zdarzeniach wymagających powiadomienia pasażerów.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// Typ incydentu
#[derive(Debug, Clone, Copy, PartialEq, Eq, sqlx::Type, Serialize, Deserialize)]
#[sqlx(type_name = "incident_type", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum IncidentType {
    Breakdown,   // Awaria techniczna
    Accident,    // Wypadek
    Delay,       // Opóźnienie
    RouteChange, // Zmiana trasy
    Other,       // Inne
}

impl IncidentType {
    pub fn to_polish(&self) -> &'static str {
        match self {
            IncidentType::Breakdown => "Awaria techniczna",
            IncidentType::Accident => "Wypadek",
            IncidentType::Delay => "Opóźnienie",
            IncidentType::RouteChange => "Zmiana trasy",
            IncidentType::Other => "Inne",
        }
    }

    pub fn to_english(&self) -> &'static str {
        match self {
            IncidentType::Breakdown => "Breakdown",
            IncidentType::Accident => "Accident",
            IncidentType::Delay => "Delay",
            IncidentType::RouteChange => "Route Change",
            IncidentType::Other => "Other",
        }
    }

    /// Czy ten typ incydentu wymaga pojazdu zastępczego
    pub fn requires_replacement(&self) -> bool {
        matches!(self, IncidentType::Breakdown | IncidentType::Accident)
    }
}

/// Poziom ważności incydentu
#[derive(Debug, Clone, Copy, PartialEq, Eq, sqlx::Type, Serialize, Deserialize, Default)]
#[sqlx(type_name = "incident_severity", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    #[default]
    Low, // Niski - nie wpływa na rozkład
    Medium,   // Średni - lekkie opóźnienie
    High,     // Wysoki - poważne opóźnienie
    Critical, // Krytyczny - pojazd wyłączony z ruchu
}

impl Severity {
    pub fn to_polish(&self) -> &'static str {
        match self {
            Severity::Low => "Niski",
            Severity::Medium => "Średni",
            Severity::High => "Wysoki",
            Severity::Critical => "Krytyczny",
        }
    }

    pub fn color(&self) -> &'static str {
        match self {
            Severity::Low => "#10B981",      // Green
            Severity::Medium => "#F59E0B",   // Yellow
            Severity::High => "#EF4444",     // Red
            Severity::Critical => "#7C3AED", // Purple
        }
    }

    /// Czy wymaga natychmiastowej uwagi
    pub fn is_urgent(&self) -> bool {
        matches!(self, Severity::High | Severity::Critical)
    }
}

/// Status incydentu
#[derive(Debug, Clone, Copy, PartialEq, Eq, sqlx::Type, Serialize, Deserialize)]
#[sqlx(type_name = "incident_status", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum IncidentStatus {
    Reported,   // Zgłoszony
    InProgress, // W trakcie rozwiązywania
    Resolved,   // Rozwiązany
    Cancelled,  // Anulowany
}

impl IncidentStatus {
    pub fn to_polish(&self) -> &'static str {
        match self {
            IncidentStatus::Reported => "Zgłoszony",
            IncidentStatus::InProgress => "W trakcie",
            IncidentStatus::Resolved => "Rozwiązany",
            IncidentStatus::Cancelled => "Anulowany",
        }
    }

    pub fn is_active(&self) -> bool {
        matches!(self, IncidentStatus::Reported | IncidentStatus::InProgress)
    }
}

/// Model incydentu w bazie danych
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct Incident {
    pub id: Uuid,
    /// ID pojazdu
    pub vehicle_id: Uuid,
    /// ID kierowcy zgłaszającego
    pub driver_id: Uuid,
    /// Typ incydentu
    pub incident_type: IncidentType,
    /// Poziom ważności
    pub severity: Severity,
    /// Tytuł/krótki opis
    pub title: String,
    /// Szczegółowy opis
    pub description: Option<String>,
    /// Lokalizacja zdarzenia (WKT format)
    pub location: Option<String>,
    /// Czas zgłoszenia
    pub reported_at: DateTime<Utc>,
    /// Czas rozwiązania
    pub resolved_at: Option<DateTime<Utc>>,
    /// Kto rozwiązał
    pub resolved_by: Option<Uuid>,
    /// Notatki z rozwiązania
    pub resolution_notes: Option<String>,
    /// Status
    pub status: IncidentStatus,
    /// ID pojazdu zastępczego
    pub replacement_vehicle_id: Option<Uuid>,
    /// Szacowany czas rozwiązania
    pub estimated_resolution: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Response API dla incydentu
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IncidentResponse {
    pub id: Uuid,
    pub vehicle_id: Uuid,
    pub vehicle_info: IncidentVehicleInfo,
    pub driver_id: Uuid,
    pub driver_name: String,
    pub incident_type: IncidentType,
    pub incident_type_name_pl: String,
    pub severity: Severity,
    pub severity_name_pl: String,
    pub severity_color: String,
    pub title: String,
    pub description: Option<String>,
    pub location: Option<IncidentLocation>,
    pub reported_at: DateTime<Utc>,
    pub resolved_at: Option<DateTime<Utc>>,
    pub resolved_by_name: Option<String>,
    pub resolution_notes: Option<String>,
    pub status: IncidentStatus,
    pub status_name_pl: String,
    pub is_active: bool,
    pub replacement_vehicle: Option<ReplacementVehicleInfo>,
    pub estimated_resolution: Option<DateTime<Utc>>,
    pub affected_routes: Vec<AffectedRouteInfo>,
    pub duration_minutes: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IncidentVehicleInfo {
    pub id: Uuid,
    pub registration_number: String,
    pub brand: String,
    pub model: String,
    pub current_route_id: Option<Uuid>,
    pub current_route_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IncidentLocation {
    pub latitude: f64,
    pub longitude: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplacementVehicleInfo {
    pub id: Uuid,
    pub registration_number: String,
    pub brand: String,
    pub model: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AffectedRouteInfo {
    pub id: Uuid,
    pub name: String,
    pub number: String,
    pub color: String,
}

/// Powiadomienie o incydencie dla pasażerów
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct IncidentNotification {
    pub id: Uuid,
    pub incident_id: Uuid,
    pub route_id: Uuid,
    pub message_pl: String,
    pub message_en: String,
    pub sent_at: DateTime<Utc>,
    pub affected_users_count: i32,
}

/// Request do tworzenia incydentu (z mobile app kierowcy)
#[derive(Debug, Deserialize)]
pub struct CreateIncidentRequest {
    pub vehicle_id: Uuid,
    pub incident_type: IncidentType,
    #[serde(default)]
    pub severity: Severity,
    pub title: String,
    pub description: Option<String>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub estimated_resolution_minutes: Option<i32>,
}

/// Request do aktualizacji incydentu
#[derive(Debug, Deserialize)]
pub struct UpdateIncidentRequest {
    pub title: Option<String>,
    pub description: Option<String>,
    pub severity: Option<Severity>,
    pub status: Option<IncidentStatus>,
    pub estimated_resolution: Option<DateTime<Utc>>,
}

/// Request do rozwiązania incydentu
#[derive(Debug, Deserialize)]
pub struct ResolveIncidentRequest {
    pub resolution_notes: String,
}

/// Request do przypisania pojazdu zastępczego
#[derive(Debug, Deserialize)]
pub struct AssignReplacementVehicleRequest {
    pub replacement_vehicle_id: Uuid,
}

/// Statystyki incydentów (dla dashboardu)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IncidentStats {
    pub total_incidents: i64,
    pub active_incidents: i64,
    pub resolved_today: i64,
    pub by_severity: Vec<SeverityCount>,
    pub by_type: Vec<TypeCount>,
    pub average_resolution_minutes: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeverityCount {
    pub severity: Severity,
    pub count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeCount {
    pub incident_type: IncidentType,
    pub count: i64,
}

/// Generuje automatyczną wiadomość o incydencie dla pasażerów
pub fn generate_passenger_notification(
    route_name: &str,
    route_number: &str,
    incident_type: IncidentType,
    _severity: Severity,
    estimated_delay_minutes: Option<i32>,
) -> (String, String) {
    let (pl, en) = match incident_type {
        IncidentType::Breakdown => {
            let pl = if let Some(minutes) = estimated_delay_minutes {
                format!(
                    "Awaria pojazdu na linii {} ({}). Opóźnienie ok. {} min.",
                    route_number, route_name, minutes
                )
            } else {
                format!(
                    "Awaria pojazdu na linii {} ({}). Trwają prace naprawcze.",
                    route_number, route_name
                )
            };
            let en = if let Some(minutes) = estimated_delay_minutes {
                format!(
                    "Vehicle breakdown on line {} ({}). Delay approx. {} min.",
                    route_number, route_name, minutes
                )
            } else {
                format!(
                    "Vehicle breakdown on line {} ({}). Repair work in progress.",
                    route_number, route_name
                )
            };
            (pl, en)
        }
        IncidentType::Delay => {
            let pl = if let Some(minutes) = estimated_delay_minutes {
                format!(
                    "Opóźnienie na linii {} ({}). Opóźnienie ok. {} min.",
                    route_number, route_name, minutes
                )
            } else {
                format!("Opóźnienie na linii {} ({}).", route_number, route_name)
            };
            let en = if let Some(minutes) = estimated_delay_minutes {
                format!(
                    "Delay on line {} ({}). Delay approx. {} min.",
                    route_number, route_name, minutes
                )
            } else {
                format!("Delay on line {} ({}).", route_number, route_name)
            };
            (pl, en)
        }
        IncidentType::Accident => {
            let pl = format!(
                "Incydent na linii {} ({}). Pojazd zastępczy został wysłany.",
                route_number, route_name
            );
            let en = format!(
                "Incident on line {} ({}). Replacement vehicle has been dispatched.",
                route_number, route_name
            );
            (pl, en)
        }
        IncidentType::RouteChange => {
            let pl = format!(
                "Zmiana trasy linii {} ({}). Sprawdź szczegóły w aplikacji.",
                route_number, route_name
            );
            let en = format!(
                "Route change for line {} ({}). Check details in the app.",
                route_number, route_name
            );
            (pl, en)
        }
        IncidentType::Other => {
            let pl = format!(
                "Zdarzenie na linii {} ({}). Możliwe opóźnienia.",
                route_number, route_name
            );
            let en = format!(
                "Incident on line {} ({}). Possible delays.",
                route_number, route_name
            );
            (pl, en)
        }
    };

    (pl, en)
}

/// Oblicza czas trwania incydentu w minutach
pub fn calculate_duration_minutes(
    reported_at: DateTime<Utc>,
    resolved_at: Option<DateTime<Utc>>,
) -> Option<i64> {
    resolved_at.map(|resolved| {
        let duration = resolved.signed_duration_since(reported_at);
        duration.num_minutes()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_incident_type_requires_replacement() {
        assert!(IncidentType::Breakdown.requires_replacement());
        assert!(IncidentType::Accident.requires_replacement());
        assert!(!IncidentType::Delay.requires_replacement());
        assert!(!IncidentType::RouteChange.requires_replacement());
        assert!(!IncidentType::Other.requires_replacement());
    }

    #[test]
    fn test_severity_is_urgent() {
        assert!(!Severity::Low.is_urgent());
        assert!(!Severity::Medium.is_urgent());
        assert!(Severity::High.is_urgent());
        assert!(Severity::Critical.is_urgent());
    }

    #[test]
    fn test_incident_status_is_active() {
        assert!(IncidentStatus::Reported.is_active());
        assert!(IncidentStatus::InProgress.is_active());
        assert!(!IncidentStatus::Resolved.is_active());
        assert!(!IncidentStatus::Cancelled.is_active());
    }

    #[test]
    fn test_generate_passenger_notification_breakdown() {
        let (pl, en) = generate_passenger_notification(
            "Wilanów - Centrum",
            "175",
            IncidentType::Breakdown,
            Severity::High,
            Some(15),
        );
        assert!(pl.contains("Awaria"));
        assert!(pl.contains("175"));
        assert!(pl.contains("15 min"));
        assert!(en.contains("breakdown"));
    }
}
