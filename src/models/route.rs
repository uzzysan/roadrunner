use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// Trasa autobusowa/tramwajowa
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct Route {
    pub id: Uuid,
    /// Nazwa trasy (np. "Linia 175")
    pub name: String,
    /// Numer linii (np. "175")
    pub number: String,
    /// Opis trasy (np. "Centrum - Lotnisko")
    pub description: String,
    /// Kolor linii (HEX, np. "#2563EB")
    pub color: String,
    /// Czy trasa jest aktywna
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}

/// Przystanek na trasie (z kolejnością)
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct RouteStop {
    pub id: Uuid,
    pub route_id: Uuid,
    pub stop_id: Uuid,
    /// Kolejność przystanku na trasie (1, 2, 3, ...)
    pub stop_order: i32,
    /// Nazwa przystanku (dla JOIN queries)
    #[sqlx(skip)]
    pub stop_name: Option<String>,
}

/// Response z trasą
#[derive(Debug, Serialize)]
pub struct RouteResponse {
    pub id: Uuid,
    pub name: String,
    pub number: String,
    pub description: String,
    pub color: String,
    pub is_active: bool,
    /// Liczba przystanków na trasie
    pub stops_count: Option<i64>,
}

/// Response z trasą i wszystkimi przystankami
#[derive(Debug, Serialize)]
pub struct RouteWithStopsResponse {
    #[serde(flatten)]
    pub route: RouteResponse,
    pub stops: Vec<RouteStopDetail>,
}

/// Szczegóły przystanku na trasie
#[derive(Debug, Serialize)]
pub struct RouteStopDetail {
    pub stop_id: Uuid,
    pub stop_name: String,
    pub stop_order: i32,
    pub latitude: f64,
    pub longitude: f64,
}

/// Request tworzenia trasy
#[derive(Debug, Deserialize)]
pub struct CreateRouteRequest {
    pub name: String,
    pub number: String,
    pub description: String,
    pub color: Option<String>, // domyślnie #2563EB
}

/// Request aktualizacji trasy
#[derive(Debug, Deserialize)]
pub struct UpdateRouteRequest {
    pub name: Option<String>,
    pub number: Option<String>,
    pub description: Option<String>,
    pub color: Option<String>,
    pub is_active: Option<bool>,
}

/// Request dodania przystanku do trasy
#[derive(Debug, Deserialize)]
pub struct AddStopToRouteRequest {
    pub stop_id: Uuid,
    pub stop_order: i32,
}

/// Request wyszukiwania tras
#[derive(Debug, Deserialize)]
pub struct SearchRoutesRequest {
    pub query: String,
    pub limit: Option<i64>,
}

impl From<Route> for RouteResponse {
    fn from(route: Route) -> Self {
        Self {
            id: route.id,
            name: route.name,
            number: route.number,
            description: route.description,
            color: route.color,
            is_active: route.is_active,
            stops_count: None, // Wypełniane osobno
        }
    }
}

/// Domyślny kolor dla nowych tras
pub const DEFAULT_ROUTE_COLOR: &str = "#2563EB";

/// Lista predefiniowanych kolorów dla tras
pub const ROUTE_COLORS: &[&str] = &[
    "#2563EB", // Niebieski
    "#EF4444", // Czerwony
    "#10B981", // Zielony
    "#F59E0B", // Żółty
    "#8B5CF6", // Fioletowy
    "#EC4899", // Różowy
    "#06B6D4", // Cyjan
    "#84CC16", // Limonkowy
];

/// Zwraca losowy kolor z palety
pub fn get_random_route_color() -> &'static str {
    use rand::seq::SliceRandom;
    use rand::thread_rng;

    let mut rng = thread_rng();
    ROUTE_COLORS.choose(&mut rng).unwrap_or(&DEFAULT_ROUTE_COLOR)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_route_color() {
        assert_eq!(DEFAULT_ROUTE_COLOR, "#2563EB");
    }

    #[test]
    fn test_route_colors_count() {
        assert_eq!(ROUTE_COLORS.len(), 8);
    }
}
