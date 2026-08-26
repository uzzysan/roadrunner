//! Model pojazdu (Vehicle) dla zarządzania flotą
//!
//! Przechowuje informacje o pojazdach transportowych wraz z ich
//! lokalizacją GPS, statusem i przypisaniem do kierowcy/trasy.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// Typ pojazdu
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, sqlx::Type, Serialize, Deserialize)]
#[sqlx(type_name = "vehicle_type", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum VehicleType {
    #[default]
    Bus, // Autobus miejski
    Minibus,    // Minibus
    Coach,      // Autokar
    Tram,       // Tramwaj
    Trolleybus, // Trolejbus
}

impl VehicleType {
    pub fn to_polish(&self) -> &'static str {
        match self {
            VehicleType::Bus => "Autobus",
            VehicleType::Minibus => "Minibus",
            VehicleType::Coach => "Autokar",
            VehicleType::Tram => "Tramwaj",
            VehicleType::Trolleybus => "Trolejbus",
        }
    }

    pub fn to_english(&self) -> &'static str {
        match self {
            VehicleType::Bus => "Bus",
            VehicleType::Minibus => "Minibus",
            VehicleType::Coach => "Coach",
            VehicleType::Tram => "Tram",
            VehicleType::Trolleybus => "Trolleybus",
        }
    }
}

/// Typ paliwa
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, sqlx::Type, Serialize, Deserialize)]
#[sqlx(type_name = "fuel_type", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum FuelType {
    #[default]
    Diesel, // Diesel
    Electric, // Elektryczny
    Hybrid,   // Hybrydowy
    Cng,      // CNG
    Hydrogen, // Wodór
}

impl FuelType {
    pub fn to_polish(&self) -> &'static str {
        match self {
            FuelType::Diesel => "Diesel",
            FuelType::Electric => "Elektryczny",
            FuelType::Hybrid => "Hybrydowy",
            FuelType::Cng => "CNG",
            FuelType::Hydrogen => "Wodór",
        }
    }
}

/// Status pojazdu
#[derive(Debug, Clone, Copy, PartialEq, Eq, sqlx::Type, Serialize, Deserialize)]
#[sqlx(type_name = "vehicle_status", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum VehicleStatus {
    Active,      // Aktywny - w ruchu
    Maintenance, // W serwisie
    Retired,     // Wycofany
    Broken,      // Uszkodzony
}

impl VehicleStatus {
    pub fn to_polish(&self) -> &'static str {
        match self {
            VehicleStatus::Active => "Aktywny",
            VehicleStatus::Maintenance => "W serwisie",
            VehicleStatus::Retired => "Wycofany",
            VehicleStatus::Broken => "Uszkodzony",
        }
    }

    pub fn is_operational(&self) -> bool {
        matches!(self, VehicleStatus::Active)
    }
}

/// Model pojazdu w bazie danych
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct Vehicle {
    pub id: Uuid,
    /// Numer rejestracyjny (np. "WX 12345")
    pub registration_number: String,
    /// Numer VIN
    pub vin: Option<String>,
    /// Marka pojazdu
    pub brand: String,
    /// Model pojazdu
    pub model: String,
    /// Rok produkcji
    pub year: Option<i32>,
    /// Pojemność pasażerska
    pub capacity: i32,
    /// Typ pojazdu
    pub vehicle_type: VehicleType,
    /// Typ paliwa
    pub fuel_type: FuelType,
    /// Status pojazdu
    pub status: VehicleStatus,
    /// ID urządzenia GPS
    pub gps_device_id: Option<String>,
    /// Ostatnia znana lokalizacja (WKT format)
    pub last_location: Option<String>,
    /// Czas ostatniej lokalizacji
    pub last_location_at: Option<DateTime<Utc>>,
    /// ID aktualnego kierowcy
    pub current_driver_id: Option<Uuid>,
    /// ID aktualnej trasy
    pub current_route_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Response API dla pojazdu
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VehicleResponse {
    pub id: Uuid,
    pub registration_number: String,
    pub vin: Option<String>,
    pub brand: String,
    pub model: String,
    pub year: Option<i32>,
    pub capacity: i32,
    pub vehicle_type: VehicleType,
    pub vehicle_type_name_pl: String,
    pub vehicle_type_name_en: String,
    pub fuel_type: FuelType,
    pub fuel_type_name_pl: String,
    pub status: VehicleStatus,
    pub status_name_pl: String,
    pub gps_device_id: Option<String>,
    pub last_location: Option<VehicleLocationResponse>,
    pub last_location_at: Option<DateTime<Utc>>,
    pub current_driver_id: Option<Uuid>,
    pub current_route_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

impl From<Vehicle> for VehicleResponse {
    fn from(v: Vehicle) -> Self {
        let last_location = v.last_location.as_ref().and_then(|loc| {
            parse_point_wkt(loc).map(|(lon, lat)| VehicleLocationResponse {
                latitude: lat,
                longitude: lon,
            })
        });

        Self {
            id: v.id,
            registration_number: v.registration_number,
            vin: v.vin,
            brand: v.brand,
            model: v.model,
            year: v.year,
            capacity: v.capacity,
            vehicle_type: v.vehicle_type,
            vehicle_type_name_pl: v.vehicle_type.to_polish().to_string(),
            vehicle_type_name_en: v.vehicle_type.to_english().to_string(),
            fuel_type: v.fuel_type,
            fuel_type_name_pl: v.fuel_type.to_polish().to_string(),
            status: v.status,
            status_name_pl: v.status.to_polish().to_string(),
            gps_device_id: v.gps_device_id,
            last_location,
            last_location_at: v.last_location_at,
            current_driver_id: v.current_driver_id,
            current_route_id: v.current_route_id,
            created_at: v.created_at,
        }
    }
}

/// Lokalizacja pojazdu
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VehicleLocationResponse {
    pub latitude: f64,
    pub longitude: f64,
}

/// Szczegółowa lokalizacja z dodatkowymi danymi
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VehicleLocationDetail {
    pub vehicle_id: Uuid,
    pub registration_number: String,
    pub route_id: Option<Uuid>,
    pub route_name: Option<String>,
    pub driver_id: Option<Uuid>,
    pub driver_name: Option<String>,
    pub latitude: f64,
    pub longitude: f64,
    pub speed: Option<f64>,
    pub heading: Option<f64>,
    pub next_stop_id: Option<Uuid>,
    pub next_stop_name: Option<String>,
    pub eta_seconds: Option<i32>,
    pub recorded_at: DateTime<Utc>,
}

/// Request do tworzenia pojazdu
#[derive(Debug, Deserialize)]
pub struct CreateVehicleRequest {
    pub registration_number: String,
    pub vin: Option<String>,
    pub brand: String,
    pub model: String,
    pub year: Option<i32>,
    #[serde(default = "default_capacity")]
    pub capacity: i32,
    #[serde(default)]
    pub vehicle_type: VehicleType,
    #[serde(default)]
    pub fuel_type: FuelType,
    pub gps_device_id: Option<String>,
}

fn default_capacity() -> i32 {
    50
}

/// Request do aktualizacji pojazdu
#[derive(Debug, Deserialize)]
pub struct UpdateVehicleRequest {
    pub registration_number: Option<String>,
    pub vin: Option<String>,
    pub brand: Option<String>,
    pub model: Option<String>,
    pub year: Option<i32>,
    pub capacity: Option<i32>,
    pub vehicle_type: Option<VehicleType>,
    pub fuel_type: Option<FuelType>,
    pub status: Option<VehicleStatus>,
    pub gps_device_id: Option<String>,
}

/// Request do aktualizacji lokalizacji (z GPS)
#[derive(Debug, Deserialize)]
pub struct UpdateVehicleLocationRequest {
    pub latitude: f64,
    pub longitude: f64,
    pub speed: Option<f64>,
    pub heading: Option<f64>,
    pub next_stop_id: Option<Uuid>,
    pub eta_seconds: Option<i32>,
}

/// Request do przypisania kierowcy
#[derive(Debug, Deserialize)]
pub struct AssignDriverRequest {
    pub driver_id: Uuid,
}

/// Request do przypisania trasy
#[derive(Debug, Deserialize)]
pub struct AssignRouteRequest {
    pub route_id: Uuid,
}

/// Parsuje WKT POINT na współrzędne
fn parse_point_wkt(wkt: &str) -> Option<(f64, f64)> {
    let point_part = if let Some(idx) = wkt.find("POINT") {
        &wkt[idx..]
    } else {
        wkt
    };

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

/// Tworzy WKT POINT z współrzędnych
pub fn make_point(longitude: f64, latitude: f64) -> String {
    format!("SRID=4326;POINT({} {})", longitude, latitude)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vehicle_status_is_operational() {
        assert!(VehicleStatus::Active.is_operational());
        assert!(!VehicleStatus::Maintenance.is_operational());
        assert!(!VehicleStatus::Broken.is_operational());
        assert!(!VehicleStatus::Retired.is_operational());
    }

    #[test]
    fn test_vehicle_type_names() {
        assert_eq!(VehicleType::Bus.to_polish(), "Autobus");
        assert_eq!(VehicleType::Bus.to_english(), "Bus");
    }

    #[test]
    fn test_make_and_parse_point() {
        let point = make_point(21.0122, 52.2297);
        let (lon, lat) = parse_point_wkt(&point).unwrap();
        assert_eq!(lon, 21.0122);
        assert_eq!(lat, 52.2297);
    }
}
