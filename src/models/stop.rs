use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// Przystanek autobusowy/tramwajowy
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct Stop {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    /// Współrzędne geograficzne (PostGIS Point)
    /// Format: "SRID=4326;POINT(lon lat)"
    pub location: String,
    pub address: Option<String>,
    /// Udogodnienia: ["shelter", "bench", "timetable", "wheelchair"]
    pub amenities: Option<Vec<String>>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}

/// Przystanek z odległością (dla zapytań "najbliższe")
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StopWithDistance {
    #[serde(flatten)]
    pub stop: Stop,
    pub distance_meters: f64,
}

/// Request tworzenia przystanku
#[derive(Debug, Deserialize)]
pub struct CreateStopRequest {
    pub name: String,
    pub description: Option<String>,
    pub latitude: f64,
    pub longitude: f64,
    pub address: Option<String>,
    pub amenities: Option<Vec<String>>,
}

/// Request aktualizacji przystanku
#[derive(Debug, Deserialize)]
pub struct UpdateStopRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub address: Option<String>,
    pub amenities: Option<Vec<String>>,
    pub is_active: Option<bool>,
}

/// Response z przystankiem
#[derive(Debug, Serialize)]
pub struct StopResponse {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub latitude: f64,
    pub longitude: f64,
    pub address: Option<String>,
    pub amenities: Option<Vec<String>>,
    pub is_active: bool,
}

/// Request wyszukiwania przystanków
#[derive(Debug, Deserialize)]
pub struct SearchStopsRequest {
    pub query: String,
    pub limit: Option<i64>,
}

/// Request najbliższych przystanków
#[derive(Debug, Deserialize)]
pub struct NearbyStopsRequest {
    pub latitude: f64,
    pub longitude: f64,
    pub radius_meters: Option<f64>, // domyślnie 1000m
    pub limit: Option<i64>, // domyślnie 10
}

impl From<Stop> for StopResponse {
    fn from(stop: Stop) -> Self {
        // Parsowanie współrzędnych z formatu PostGIS
        let (lon, lat) = parse_point(&stop.location).unwrap_or((0.0, 0.0));

        Self {
            id: stop.id,
            name: stop.name,
            description: stop.description,
            latitude: lat,
            longitude: lon,
            address: stop.address,
            amenities: stop.amenities,
            is_active: stop.is_active,
        }
    }
}

/// Parsuje punkt PostGIS "SRID=4326;POINT(lon lat)"
fn parse_point(point_str: &str) -> Option<(f64, f64)> {
    // Usuń prefix SRID
    let coords = point_str.split("POINT(").nth(1)?;
    let coords = coords.trim_end_matches(")");
    let parts: Vec<&str> = coords.split_whitespace().collect();

    if parts.len() != 2 {
        return None;
    }

    let lon = parts[0].parse().ok()?;
    let lat = parts[1].parse().ok()?;

    Some((lon, lat))
}

/// Tworzy string PostGIS Point
pub fn make_point(longitude: f64, latitude: f64) -> String {
    format!("SRID=4326;POINT({} {})", longitude, latitude)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_point() {
        let point = "SRID=4326;POINT(21.0122 52.2297)";
        let (lon, lat) = parse_point(point).unwrap();
        assert_eq!(lon, 21.0122);
        assert_eq!(lat, 52.2297);
    }

    #[test]
    fn test_make_point() {
        let point = make_point(21.0122, 52.2297);
        assert_eq!(point, "SRID=4326;POINT(21.0122 52.2297)");
    }
}
