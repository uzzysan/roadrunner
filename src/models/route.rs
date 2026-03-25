//! Model trasy linii autobusowej (Route)
//!
//! Przechowuje informacje o liniach autobusowych wraz z ich kolorystyką.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// Paleta kolorów dla linii autobusowych
/// Kolory są dobrane dla dobrej widoczności na mapie i w interfejsie
pub const ROUTE_COLORS: &[&str] = &[
    "#2563EB", // Niebieski
    "#EF4444", // Czerwony
    "#10B981", // Zielony
    "#F59E0B", // Żółty/Pomarańczowy
    "#8B5CF6", // Fioletowy
    "#EC4899", // Różowy
    "#06B6D4", // Cyjan
    "#84CC16", // Limonkowy
    "#F97316", // Pomarańczowy
    "#6366F1", // Indygo
    "#14B8A6", // Teal
    "#EAB308", // Żółty
];

/// Zwraca kolor dla linii na podstawie numeru
/// Używa hashowania dla deterministycznego przypisania koloru
pub fn get_route_color(number: &str) -> &'static str {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    number.hash(&mut hasher);
    let hash = hasher.finish();

    ROUTE_COLORS[(hash as usize) % ROUTE_COLORS.len()]
}

/// Model trasy w bazie danych
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct Route {
    pub id: Uuid,
    /// Pełna nazwa linii np. "Linia 175"
    pub name: String,
    /// Numer linii np. "175", "N01", "Z-2"
    pub number: String,
    /// Opis trasy np. "Wilanów - Plac Wilsona"
    pub description: String,
    /// Kolor linii w formacie HEX np. "#2563EB"
    pub color: String,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}

/// Response API dla trasy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteResponse {
    pub id: Uuid,
    pub name: String,
    pub number: String,
    pub description: String,
    pub color: String,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
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
            created_at: route.created_at,
        }
    }
}

/// Trasa z listą przystanków
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteWithStops {
    #[serde(flatten)]
    pub route: RouteResponse,
    pub stops: Vec<StopInRoute>,
}

/// Przystanek w ramach trasy
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct StopInRoute {
    pub id: Uuid,
    pub name: String,
    pub longitude: f64,
    pub latitude: f64,
    /// Kolejność przystanku w trasie
    pub stop_order: i32,
    /// Czy przystanek jest opcjonalny (na żądanie)
    pub is_optional: bool,
}

/// Request do tworzenia nowej trasy
#[derive(Debug, Deserialize)]
pub struct CreateRouteRequest {
    pub name: String,
    pub number: String,
    pub description: String,
    /// Opcjonalny kolor (jeśli nie podany, zostanie wygenerowany)
    pub color: Option<String>,
}

/// Request do aktualizacji trasy
#[derive(Debug, Deserialize)]
pub struct UpdateRouteRequest {
    pub name: Option<String>,
    pub number: Option<String>,
    pub description: Option<String>,
    pub color: Option<String>,
    pub is_active: Option<bool>,
}

/// Request do dodania przystanku do trasy
#[derive(Debug, Deserialize)]
pub struct AddStopToRouteRequest {
    pub stop_id: Uuid,
    pub stop_order: i32,
    pub is_optional: Option<bool>,
}

/// Request do aktualizacji kolejności przystanków
#[derive(Debug, Deserialize)]
pub struct ReorderStopsRequest {
    pub stop_orders: Vec<StopOrder>,
}

#[derive(Debug, Deserialize)]
pub struct StopOrder {
    pub stop_id: Uuid,
    pub new_order: i32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_route_color_deterministic() {
        // Ten sam numer zawsze zwraca ten sam kolor
        let color1 = get_route_color("175");
        let color2 = get_route_color("175");
        assert_eq!(color1, color2);
    }

    #[test]
    fn test_get_route_color_different() {
        // Różne numery mogą (ale nie muszą) mieć różne kolory
        let color1 = get_route_color("175");
        let color2 = get_route_color("N01");
        // W praktyce prawdopodobieństwo kolizji jest małe dla 12 kolorów
        // ale nie możemy tego zagwarantować w teście
        assert!(!color1.is_empty());
        assert!(!color2.is_empty());
    }

    #[test]
    fn test_route_colors_valid_hex() {
        for color in ROUTE_COLORS {
            assert!(color.starts_with('#'));
            assert_eq!(color.len(), 7); // #RRGGBB
        }
    }
}
