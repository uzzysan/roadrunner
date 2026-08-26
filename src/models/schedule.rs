//! Model rozkładu jazdy (Schedule)
//!
//! Przechowuje informacje o odjazdach autobusów z przystanków.

use chrono::{DateTime, NaiveTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// Typ dnia dla rozkładu jazdy
#[derive(Debug, Clone, Copy, PartialEq, Eq, sqlx::Type, Serialize, Deserialize)]
#[sqlx(type_name = "day_type", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum DayType {
    /// Dni robocze (pon-pt)
    Weekday,
    /// Soboty
    Saturday,
    /// Niedziele
    Sunday,
    /// Święta
    Holiday,
    /// Codziennie
    Everyday,
}

impl DayType {
    /// Zwraca nazwę dnia w języku polskim
    pub fn to_polish(&self) -> &'static str {
        match self {
            DayType::Weekday => "Dni robocze",
            DayType::Saturday => "Soboty",
            DayType::Sunday => "Niedziele",
            DayType::Holiday => "Święta",
            DayType::Everyday => "Codziennie",
        }
    }

    /// Zwraca nazwę dnia w języku angielskim
    pub fn to_english(&self) -> &'static str {
        match self {
            DayType::Weekday => "Weekdays",
            DayType::Saturday => "Saturdays",
            DayType::Sunday => "Sundays",
            DayType::Holiday => "Holidays",
            DayType::Everyday => "Every day",
        }
    }

    /// Parsuje string na DayType
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "weekday" | "weekdays" | "robocze" | "dni robocze" => Some(DayType::Weekday),
            "saturday" | "saturdays" | "sobota" | "soboty" => Some(DayType::Saturday),
            "sunday" | "sundays" | "niedziela" | "niedziele" => Some(DayType::Sunday),
            "holiday" | "holidays" | "święto" | "święta" => Some(DayType::Holiday),
            "everyday" | "every day" | "codziennie" | "daily" => Some(DayType::Everyday),
            _ => None,
        }
    }
}

impl std::fmt::Display for DayType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

/// Model rozkładu jazdy w bazie danych
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct Schedule {
    pub id: Uuid,
    pub route_id: Uuid,
    pub stop_id: Uuid,
    /// Czas przyjazdu (dla przystanków pośrednich)
    pub arrival_time: NaiveTime,
    /// Czas odjazdu
    pub departure_time: NaiveTime,
    /// Typ dnia
    pub day_type: DayType,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}

/// Rozkład z informacjami o trasie
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct ScheduleWithRoute {
    pub id: Uuid,
    pub route_id: Uuid,
    pub stop_id: Uuid,
    pub arrival_time: NaiveTime,
    pub departure_time: NaiveTime,
    #[sqlx(rename = "day_type")]
    pub day_type: DayType,
    pub is_active: bool,
    pub route_name: String,
    pub route_number: String,
    pub route_color: String,
    pub route_description: Option<String>,
}

/// Rozkład z informacjami o przystanku
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct ScheduleWithStop {
    pub id: Uuid,
    pub route_id: Uuid,
    pub stop_id: Uuid,
    pub arrival_time: NaiveTime,
    pub departure_time: NaiveTime,
    #[sqlx(rename = "day_type")]
    pub day_type: DayType,
    pub is_active: bool,
    pub stop_name: String,
    pub stop_latitude: f64,
    pub stop_longitude: f64,
}

/// Response API dla rozkładu
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduleResponse {
    pub id: Uuid,
    pub route_id: Uuid,
    pub stop_id: Uuid,
    pub arrival_time: String,
    pub departure_time: String,
    pub day_type: DayType,
    pub day_type_name_pl: String,
    pub day_type_name_en: String,
    pub is_active: bool,
}

impl From<Schedule> for ScheduleResponse {
    fn from(schedule: Schedule) -> Self {
        Self {
            id: schedule.id,
            route_id: schedule.route_id,
            stop_id: schedule.stop_id,
            arrival_time: schedule.arrival_time.format("%H:%M").to_string(),
            departure_time: schedule.departure_time.format("%H:%M").to_string(),
            day_type: schedule.day_type,
            day_type_name_pl: schedule.day_type.to_polish().to_string(),
            day_type_name_en: schedule.day_type.to_english().to_string(),
            is_active: schedule.is_active,
        }
    }
}

/// Request do tworzenia nowego rozkładu
#[derive(Debug, Deserialize)]
pub struct CreateScheduleRequest {
    pub route_id: Uuid,
    pub stop_id: Uuid,
    /// Czas w formacie "HH:MM" lub "HH:MM:SS"
    pub arrival_time: String,
    /// Czas w formacie "HH:MM" lub "HH:MM:SS"
    pub departure_time: String,
    pub day_type: DayType,
}

/// Request do tworzenia wielu rozkładów (batch)
#[derive(Debug, Deserialize)]
pub struct CreateSchedulesBatchRequest {
    pub route_id: Uuid,
    pub schedules: Vec<SingleScheduleRequest>,
}

#[derive(Debug, Deserialize)]
pub struct SingleScheduleRequest {
    pub stop_id: Uuid,
    pub arrival_time: String,
    pub departure_time: String,
    pub day_type: DayType,
}

/// Request do aktualizacji rozkładu
#[derive(Debug, Deserialize)]
pub struct UpdateScheduleRequest {
    pub arrival_time: Option<String>,
    pub departure_time: Option<String>,
    pub day_type: Option<DayType>,
    pub is_active: Option<bool>,
}

/// Request do filtrowania rozkładów
#[derive(Debug, Deserialize)]
pub struct FilterSchedulesRequest {
    pub route_id: Option<Uuid>,
    pub stop_id: Option<Uuid>,
    pub day_type: Option<DayType>,
    pub from_time: Option<String>,
    pub to_time: Option<String>,
}

/// Parsuje czas z stringa
/// Akceptuje formaty: "HH:MM" i "HH:MM:SS"
pub fn parse_time(time_str: &str) -> Result<NaiveTime, String> {
    // Spróbuj "HH:MM:SS"
    if let Ok(time) = NaiveTime::parse_from_str(time_str, "%H:%M:%S") {
        return Ok(time);
    }

    // Spróbuj "HH:MM"
    if let Ok(time) = NaiveTime::parse_from_str(time_str, "%H:%M") {
        return Ok(time);
    }

    Err(format!(
        "Nieprawidłowy format czasu: {}. Użyj HH:MM lub HH:MM:SS",
        time_str
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Timelike;

    #[test]
    fn test_day_type_to_polish() {
        assert_eq!(DayType::Weekday.to_polish(), "Dni robocze");
        assert_eq!(DayType::Saturday.to_polish(), "Soboty");
        assert_eq!(DayType::Sunday.to_polish(), "Niedziele");
        assert_eq!(DayType::Holiday.to_polish(), "Święta");
        assert_eq!(DayType::Everyday.to_polish(), "Codziennie");
    }

    #[test]
    fn test_day_type_to_english() {
        assert_eq!(DayType::Weekday.to_english(), "Weekdays");
        assert_eq!(DayType::Saturday.to_english(), "Saturdays");
        assert_eq!(DayType::Sunday.to_english(), "Sundays");
        assert_eq!(DayType::Holiday.to_english(), "Holidays");
        assert_eq!(DayType::Everyday.to_english(), "Every day");
    }

    #[test]
    fn test_day_type_from_str() {
        assert_eq!(DayType::from_str("weekday"), Some(DayType::Weekday));
        assert_eq!(DayType::from_str("WEEKDAY"), Some(DayType::Weekday));
        assert_eq!(DayType::from_str("robocze"), Some(DayType::Weekday));
        assert_eq!(DayType::from_str("dni robocze"), Some(DayType::Weekday));
        assert_eq!(DayType::from_str("sobota"), Some(DayType::Saturday));
        assert_eq!(DayType::from_str("codziennie"), Some(DayType::Everyday));
        assert_eq!(DayType::from_str("invalid"), None);
    }

    #[test]
    fn test_parse_time_hh_mm() {
        let time = parse_time("14:30").unwrap();
        assert_eq!(time.hour(), 14);
        assert_eq!(time.minute(), 30);
        assert_eq!(time.second(), 0);
    }

    #[test]
    fn test_parse_time_hh_mm_ss() {
        let time = parse_time("14:30:45").unwrap();
        assert_eq!(time.hour(), 14);
        assert_eq!(time.minute(), 30);
        assert_eq!(time.second(), 45);
    }

    #[test]
    fn test_parse_time_invalid() {
        assert!(parse_time("invalid").is_err());
        assert!(parse_time("25:00").is_err());
        assert!(parse_time("14:70").is_err());
    }
}
