//! Model przystanku (Stop) z obsługą PostGIS
//!
//! Przechowuje informacje o przystankach autobusowych wraz z lokalizacją geograficzną.
//! Używa PostGIS do przechowywania i wyszukiwania lokalizacji.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// Model przystanku w bazie danych
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct Stop {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    /// Lokalizacja w formacie WKT: "SRID=4326;POINT(lon lat)"
    pub location: String,
    pub address: Option<String>,
    /// Udogodnienia: wiaty, ławki, monitoring, itp.
    pub amenities: Option<Vec<String>>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}

/// Response API dla przystanku z sparsowanymi współrzędnymi
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StopResponse {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub latitude: f64,
    pub longitude: f64,
    pub address: Option<String>,
    pub amenities: Option<Vec<String>>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}

impl From<Stop> for StopResponse {
    fn from(stop: Stop) -> Self {
        let (lon, lat) = parse_point_wkt(&stop.location)
            .unwrap_or((0.0, 0.0));

        Self {
            id: stop.id,
            name: stop.name,
            description: stop.description,
            latitude: lat,
            longitude: lon,
            address: stop.address,
            amenities: stop.amenities,
            is_active: stop.is_active,
            created_at: stop.created_at,
        }
    }
}

/// Tworzy string WKT POINT dla PostGIS
/// 
/// # Arguments
/// * `longitude` - Długość geograficzna (-180 do 180)
/// * `latitude` - Szerokość geograficzna (-90 do 90)
/// 
/// # Returns
/// String w formacie "SRID=4326;POINT(lon lat)"
/// 
/// # Example
/// ```
/// let point = make_point(21.0118, 52.2297); // Warszawa
/// assert_eq!(point, "SRID=4326;POINT(21.0118 52.2297)");
/// ```
pub fn make_point(longitude: f64, latitude: f64) -> String {
    format!("SRID=4326;POINT({} {})", longitude, latitude)
}

/// Parsuje string WKT POINT na współrzędne (lon, lat)
/// 
/// # Arguments
/// * `wkt` - String w formacie "SRID=4326;POINT(lon lat)" lub "POINT(lon lat)"
/// 
/// # Returns
/// Option<(longitude, latitude)>
/// 
/// # Example
/// ```
/// let (lon, lat) = parse_point_wkt("SRID=4326;POINT(21.0118 52.2297)").unwrap();
/// assert_eq!(lon, 21.0118);
/// assert_eq!(lat, 52.2297);
/// ```
pub fn parse_point_wkt(wkt: &str) -> Option<(f64, f64)> {
    // Usuń prefix SRID
    let point_part = if let Some(idx) = wkt.find("POINT") {
        &wkt[idx..]
    } else {
        wkt
    };

    // Ekstrakcja współrzędnych z "POINT(lon lat)"
    let coords_start = point_part.find('(')?;
    let coords_end = point_part.find(')')?;
    let coords = &point_part[coords_start + 1..coords_end];
    
    let parts: Vec<&str> = coords.split_whitespace().collect();
    if parts.len() != 2 {
        return None;
    }

    let lon = parts[0].parse::<f64>().ok()?;
    let lat = parts[1].parse::<f64>().ok()?;

    Some((lon, lat))
}

/// Request do tworzenia nowego przystanku
#[derive(Debug, Deserialize)]
pub struct CreateStopRequest {
    pub name: String,
    pub description: Option<String>,
    pub longitude: f64,
    pub latitude: f64,
    pub address: Option<String>,
    pub amenities: Option<Vec<String>>,
}

/// Request do aktualizacji przystanku
#[derive(Debug, Deserialize)]
pub struct UpdateStopRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub longitude: Option<f64>,
    pub latitude: Option<f64>,
    pub address: Option<String>,
    pub amenities: Option<Vec<String>>,
    pub is_active: Option<bool>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_make_point() {
        let point = make_point(21.0118, 52.2297);
        assert_eq!(point, "SRID=4326;POINT(21.0118 52.2297)");
    }

    #[test]
    fn test_parse_point_wkt_with_srid() {
        let (lon, lat) = parse_point_wkt("SRID=4326;POINT(21.0118 52.2297)").unwrap();
        assert_eq!(lon, 21.0118);
        assert_eq!(lat, 52.2297);
    }

    #[test]
    fn test_parse_point_wkt_without_srid() {
        let (lon, lat) = parse_point_wkt("POINT(21.0118 52.2297)").unwrap();
        assert_eq!(lon, 21.0118);
        assert_eq!(lat, 52.2297);
    }

    #[test]
    fn test_parse_point_wkt_negative_coords() {
        let (lon, lat) = parse_point_wkt("POINT(-122.4194 37.7749)").unwrap();
        assert_eq!(lon, -122.4194);
        assert_eq!(lat, 37.7749);
    }
}
