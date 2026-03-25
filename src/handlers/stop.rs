//! Handler dla przystanków (stops) z obsługą PostGIS
//!
//! Endpointy:
//! - GET /stops - lista wszystkich przystanków
//! - GET /stops/nearby?lat={}&lon={}&radius={} - najbliższe przystanki
//! - GET /stops/:id - szczegóły przystanku
//! - GET /stops/:id/schedules - rozkład jazdy dla przystanku
//! - POST /stops/search - wyszukiwanie przystanków po nazwie

use axum::{
    extract::{Path, Query, State},
    Json,
};
use serde::{Deserialize, Serialize};
use sqlx::Row;
use uuid::Uuid;

use crate::{
    models::stop::{Stop, StopResponse},
    models::schedule::{Schedule, ScheduleWithRoute},
    state::AppState,
    errors::AppError,
};

/// Query parameters dla wyszukiwania pobliskich przystanków
#[derive(Debug, Deserialize)]
pub struct NearbyQuery {
    /// Szerokość geograficzna
    pub lat: f64,
    /// Długość geograficzna
    pub lon: f64,
    /// Promień w metrach (domyślnie 500m)
    #[serde(default = "default_radius")]
    pub radius: i32,
}

fn default_radius() -> i32 {
    500
}

/// Request dla wyszukiwania przystanków
#[derive(Debug, Deserialize)]
pub struct SearchStopsRequest {
    pub query: String,
    #[serde(default = "default_limit")]
    pub limit: i32,
}

fn default_limit() -> i32 {
    20
}

/// Response dla listy przystanków
#[derive(Debug, Serialize)]
pub struct StopsListResponse {
    pub stops: Vec<StopResponse>,
    pub total: i64,
}

/// Response dla pobliskich przystanków z odległością
#[derive(Debug, Serialize)]
pub struct NearbyStopResponse {
    #[serde(flatten)]
    pub stop: StopResponse,
    /// Odległość w metrach
    pub distance_meters: f64,
}

/// GET /stops - lista wszystkich aktywnych przystanków
pub async fn list_stops(
    State(state): State<AppState>,
) -> Result<Json<StopsListResponse>, AppError> {
    let stops = sqlx::query_as::<_, Stop>(
        r#"
        SELECT 
            id,
            name,
            description,
            ST_AsText(location) as location,
            address,
            amenities,
            is_active,
            created_at
        FROM stops
        WHERE is_active = true
        ORDER BY name
        "#
    )
    .fetch_all(&state.db)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    let total = stops.len() as i64;
    let stop_responses: Vec<StopResponse> = stops.into_iter()
        .map(StopResponse::from)
        .collect();

    Ok(Json(StopsListResponse {
        stops: stop_responses,
        total,
    }))
}

/// GET /stops/nearby?lat={}&lon={}&radius={} - pobliskie przystanki
/// 
/// Używa PostGIS do obliczenia odległości i znalezienia przystanków w promieniu
pub async fn nearby_stops(
    State(state): State<AppState>,
    Query(query): Query<NearbyQuery>,
) -> Result<Json<Vec<NearbyStopResponse>>, AppError> {
    // Walidacja współrzędnych
    if query.lat < -90.0 || query.lat > 90.0 {
        return Err(AppError::ValidationError(
            "Szerokość geograficzna musi być między -90 a 90".to_string()
        ));
    }
    if query.lon < -180.0 || query.lon > 180.0 {
        return Err(AppError::ValidationError(
            "Długość geograficzna musi być między -180 a 180".to_string()
        ));
    }

    let rows = sqlx::query(
        r#"
        SELECT 
            id,
            name,
            description,
            ST_AsText(location) as location,
            address,
            amenities,
            is_active,
            created_at,
            ST_Distance(
                location::geography,
                ST_SetSRID(ST_MakePoint($1, $2), 4326)::geography
            ) as distance
        FROM stops
        WHERE is_active = true
          AND ST_DWithin(
              location::geography,
              ST_SetSRID(ST_MakePoint($1, $2), 4326)::geography,
              $3
          )
        ORDER BY distance
        LIMIT 50
        "#
    )
    .bind(query.lon)  // PostGIS: ST_MakePoint(x, y) = (lon, lat)
    .bind(query.lat)
    .bind(query.radius)
    .fetch_all(&state.db)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    let mut nearby_stops = Vec::new();

    for row in rows {
        let stop = Stop {
            id: row.try_get("id").map_err(|e| AppError::DatabaseError(e.to_string()))?,
            name: row.try_get("name").map_err(|e| AppError::DatabaseError(e.to_string()))?,
            description: row.try_get("description").ok(),
            location: row.try_get("location").map_err(|e| AppError::DatabaseError(e.to_string()))?,
            address: row.try_get("address").ok(),
            amenities: row.try_get("amenities").ok(),
            is_active: row.try_get("is_active").map_err(|e| AppError::DatabaseError(e.to_string()))?,
            created_at: row.try_get("created_at").map_err(|e| AppError::DatabaseError(e.to_string()))?,
        };

        let distance: f64 = row.try_get("distance").map_err(|e| AppError::DatabaseError(e.to_string()))?;

        nearby_stops.push(NearbyStopResponse {
            stop: StopResponse::from(stop),
            distance_meters: distance.round(),
        });
    }

    Ok(Json(nearby_stops))
}

/// GET /stops/:id - szczegóły przystanku
pub async fn get_stop(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<StopResponse>, AppError> {
    let stop = sqlx::query_as::<_, Stop>(
        r#"
        SELECT 
            id,
            name,
            description,
            ST_AsText(location) as location,
            address,
            amenities,
            is_active,
            created_at
        FROM stops
        WHERE id = $1 AND is_active = true
        "#
    )
    .bind(id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?
    .ok_or_else(|| AppError::NotFound(format!("Przystanek o ID {} nie istnieje", id)))?;

    Ok(Json(StopResponse::from(stop)))
}

/// GET /stops/:id/schedules - rozkład jazdy dla przystanku
/// 
/// Zwraca wszystkie odjazdy z danego przystanku pogrupowane według linii
pub async fn get_stop_schedules(
    State(state): State<AppState>,
    Path(stop_id): Path<Uuid>,
) -> Result<Json<Vec<ScheduleWithRoute>>, AppError> {
    // Sprawdź czy przystanek istnieje
    let stop_exists: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM stops WHERE id = $1 AND is_active = true)"
    )
    .bind(stop_id)
    .fetch_one(&state.db)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    if !stop_exists {
        return Err(AppError::NotFound(format!("Przystanek o ID {} nie istnieje", stop_id)));
    }

    let schedules = sqlx::query_as::<_, ScheduleWithRoute>(
        r#"
        SELECT 
            s.id,
            s.route_id,
            s.stop_id,
            s.arrival_time,
            s.departure_time,
            s.day_type as "day_type: _",
            s.is_active,
            r.name as route_name,
            r.number as route_number,
            r.color as route_color,
            r.description as route_description
        FROM schedules s
        JOIN routes r ON s.route_id = r.id
        WHERE s.stop_id = $1 
          AND s.is_active = true
          AND r.is_active = true
        ORDER BY s.departure_time
        "#
    )
    .bind(stop_id)
    .fetch_all(&state.db)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    Ok(Json(schedules))
}

/// POST /stops/search - wyszukiwanie przystanków po nazwie
pub async fn search_stops(
    State(state): State<AppState>,
    Json(request): Json<SearchStopsRequest>,
) -> Result<Json<Vec<StopResponse>>, AppError> {
    if request.query.trim().is_empty() {
        return Err(AppError::ValidationError(
            "Zapytanie wyszukiwania nie może być puste".to_string()
        ));
    }

    let search_pattern = format!("%{}%", request.query);

    let stops = sqlx::query_as::<_, Stop>(
        r#"
        SELECT 
            id,
            name,
            description,
            ST_AsText(location) as location,
            address,
            amenities,
            is_active,
            created_at
        FROM stops
        WHERE is_active = true
          AND (
              name ILIKE $1 
              OR address ILIKE $1
              OR description ILIKE $1
          )
        ORDER BY 
            CASE 
                WHEN name ILIKE $2 THEN 0
                WHEN name ILIKE $1 THEN 1
                ELSE 2
            END,
            name
        LIMIT $3
        "#
    )
    .bind(&search_pattern)
    .bind(format!("{}%", request.query))  // Exact prefix match priority
    .bind(request.limit)
    .fetch_all(&state.db)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    let stop_responses: Vec<StopResponse> = stops.into_iter()
        .map(StopResponse::from)
        .collect();

    Ok(Json(stop_responses))
}

/// GET /stops/:id/routes - linie obsługujące dany przystanek
#[derive(Debug, Serialize)]
pub struct RouteAtStop {
    pub route_id: Uuid,
    pub route_name: String,
    pub route_number: String,
    pub route_color: String,
    pub first_departure: Option<String>,
    pub last_departure: Option<String>,
}

pub async fn get_stop_routes(
    State(state): State<AppState>,
    Path(stop_id): Path<Uuid>,
) -> Result<Json<Vec<RouteAtStop>>, AppError> {
    // Sprawdź czy przystanek istnieje
    let stop_exists: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM stops WHERE id = $1 AND is_active = true)"
    )
    .bind(stop_id)
    .fetch_one(&state.db)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    if !stop_exists {
        return Err(AppError::NotFound(format!("Przystanek o ID {} nie istnieje", stop_id)));
    }

    let routes = sqlx::query_as::<_, RouteAtStop>(
        r#"
        SELECT DISTINCT
            r.id as route_id,
            r.name as route_name,
            r.number as route_number,
            r.color as route_color,
            (SELECT MIN(departure_time::text) FROM schedules 
             WHERE route_id = r.id AND stop_id = $1 AND is_active = true) as first_departure,
            (SELECT MAX(departure_time::text) FROM schedules 
             WHERE route_id = r.id AND stop_id = $1 AND is_active = true) as last_departure
        FROM routes r
        JOIN route_stops rs ON r.id = rs.route_id
        WHERE rs.stop_id = $1
          AND r.is_active = true
        ORDER BY r.number
        "#
    )
    .bind(stop_id)
    .fetch_all(&state.db)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    Ok(Json(routes))
}
