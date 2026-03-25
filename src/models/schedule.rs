use chrono::{NaiveTime};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// Typ dnia dla rozkładu jazdy
#[derive(Debug, Clone, Copy, PartialEq, Eq, sqlx::Type, Serialize, Deserialize)]
#[sqlx(type_name = "day_type", rename_all = "snake_case")]
pub enum DayType {
    /// Dni robocze (pon-pt)
    Weekday,
    /// Sobota
    Saturday,
    /// Niedziela
    Sunday,
    /// Święta
    Holiday,
    /// Cały tydzień
    Everyday,
}

impl DayType {
    /// Zwraca nazwę dnia po polsku
    pub fn name_pl(&self) -> &'static str {
        match self {
            DayType::Weekday => "Dni robocze",
            DayType::Saturday => "Sobota",
            DayType::Sunday => "Niedziela",
            DayType::Holiday => "Święta",
            DayType::Everyday => "Codziennie",
        }
    }
}

/// Rozkład jazdy (godziny przyjazdu/odjazdu)
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct Schedule {
    pub id: Uuid,
    pub route_id: Uuid,
    pub stop_id: Uuid,
    /// Godzina przyjazdu
    pub arrival_time: NaiveTime,
    /// Godzina odjazdu
    pub departure_time: NaiveTime,
    pub day_type: DayType,
    pub is_active: bool,
}

/// Response z rozkładem jazdy
#[derive(Debug, Serialize)]
pub struct ScheduleResponse {
    pub id: Uuid,
    pub route_id: Uuid,
    pub route_name: String,
    pub route_number: String,
    pub stop_id: Uuid,
    pub stop_name: String,
    pub arrival_time: String, // Format: "HH:MM"
    pub departure_time: String, // Format: "HH:MM"
    pub day_type: String,
}

/// Request tworzenia rozkładu
#[derive(Debug, Deserialize)]
pub struct CreateScheduleRequest {
    pub route_id: Uuid,
    pub stop_id: Uuid,
    /// Format: "HH:MM"
    pub arrival_time: String,
    /// Format: "HH:MM"
    pub departure_time: String,
    pub day_type: DayType,
}

/// Request aktualizacji rozkładu
#[derive(Debug, Deserialize)]
pub struct UpdateScheduleRequest {
    pub arrival_time: Option<String>,
    pub departure_time: Option<String>,
    pub day_type: Option<DayType>,
    pub is_active: Option<bool>,
}

/// Request pobierania rozkładu dla przystanku
#[derive(Debug, Deserialize)]
pub struct GetStopSchedulesRequest {
    pub stop_id: Uuid,
    pub day_type: Option<DayType>,
    pub limit: Option<i64>,
}

/// Request pobierania rozkładu dla trasy
#[derive(Debug, Deserialize)]
pub struct GetRouteSchedulesRequest {
    pub route_id: Uuid,
    pub day_type: Option<DayType>,
}

/// Parsuje string "HH:MM" do NaiveTime
pub fn parse_time(time_str: &str) -> Option<NaiveTime> {
    let parts: Vec<&str> = time_str.split(':').collect();
    if parts.len() != 2 {
        return None;
    }

    let hour = parts[0].parse::<u32>().ok()?;
    let minute = parts[1].parse::<u32>().ok()?;

    NaiveTime::from_hms_opt(hour, minute, 0)
}

/// Formatuje NaiveTime do string "HH:MM"
pub fn format_time(time: NaiveTime) -> String {
    time.format("%H:%M").to_string()
}

/// Pobiera aktualny DayType na podstawie dnia tygodnia
pub fn get_current_day_type() -> DayType {
    use chrono::Local;

    let now = Local::now();
    let weekday = now.weekday();

    match weekday {
        chrono::Weekday::Mon |
        chrono::Weekday::Tue |
        chrono::Weekday::Wed |
        chrono::Weekday::Thu |
        chrono::Weekday::Fri => DayType::Weekday,
        chrono::Weekday::Sat => DayType::Saturday,
        chrono::Weekday::Sun => DayType::Sunday,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_time() {
        let time = parse_time("14:30").unwrap();
        assert_eq!(time.hour(), 14);
        assert_eq!(time.minute(), 30);
    }

    #[test]
    fn test_format_time() {
        let time = NaiveTime::from_hms_opt(9, 15, 0).unwrap();
        assert_eq!(format_time(time), "09:15");
    }

    #[test]
    fn test_day_type_name_pl() {
        assert_eq!(DayType::Weekday.name_pl(), "Dni robocze");
        assert_eq!(DayType::Everyday.name_pl(), "Codziennie");
    }
}
