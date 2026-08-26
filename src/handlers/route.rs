//! Handler dla tras linii autobusowych (routes)
//!
//! Endpointy:
//! - GET /routes - lista wszystkich linii
//! - GET /routes/:id - szczegóły linii z przystankami
//! - GET /routes/:id/schedules - pełny rozkład jazdy linii
//! - GET /routes/:id/geometry - geometria trasy na mapie

use axum::{
    extract::{Path, Query, State},
    Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    errors::AppError,
    models::route::{Route, RouteResponse, StopInRoute},
    state::AppState,
};

/// Query parameters dla filtrowania tras
#[derive(Debug, Deserialize)]
pub struct RoutesQuery {
    /// Filtrowanie po aktywności (domyślnie true)
    #[serde(default = "default_active_only")]
    pub active_only: bool,
}

fn default_active_only() -> bool {
    true
}

/// Response dla listy tras
#[derive(Debug, Serialize)]
pub struct RoutesListResponse {
    pub routes: Vec<RouteResponse>,
    pub total: i64,
}

/// Response dla szczegółów trasy z przystankami
#[derive(Debug, Serialize)]
pub struct RouteDetailResponse {
    #[serde(flatten)]
    pub route: RouteResponse,
    pub stops: Vec<StopInRoute>,
    pub stops_count: i32,
}

/// GET /routes - lista wszystkich linii
pub async fn list_routes(
    State(state): State<AppState>,
    Query(query): Query<RoutesQuery>,
) -> Result<Json<RoutesListResponse>, AppError> {
    let routes = if query.active_only {
        sqlx::query_as::<_, Route>(
            r#"
            SELECT 
                id,
                name,
                number,
                description,
                color,
                is_active,
                created_at
            FROM routes
            WHERE is_active = true
            ORDER BY 
                CASE 
                    WHEN number ~ '^[0-9]+$' THEN number::int
                    ELSE 999999
                END,
                number
            "#,
        )
        .fetch_all(&state.db)
        .await
    } else {
        sqlx::query_as::<_, Route>(
            r#"
            SELECT 
                id,
                name,
                number,
                description,
                color,
                is_active,
                created_at
            FROM routes
            ORDER BY 
                CASE 
                    WHEN number ~ '^[0-9]+$' THEN number::int
                    ELSE 999999
                END,
                number
            "#,
        )
        .fetch_all(&state.db)
        .await
    }
    .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    let total = routes.len() as i64;
    let route_responses: Vec<RouteResponse> = routes.into_iter().map(RouteResponse::from).collect();

    Ok(Json(RoutesListResponse {
        routes: route_responses,
        total,
    }))
}

/// GET /routes/:id - szczegóły linii z listą przystanków
pub async fn get_route(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<RouteDetailResponse>, AppError> {
    // Pobierz trasę
    let route = sqlx::query_as::<_, Route>(
        r#"
        SELECT 
            id,
            name,
            number,
            description,
            color,
            is_active,
            created_at
        FROM routes
        WHERE id = $1 AND is_active = true
        "#,
    )
    .bind(id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?
    .ok_or_else(|| AppError::NotFound(format!("Linia o ID {} nie istnieje", id)))?;

    // Pobierz przystanki dla tej trasy
    let stops = sqlx::query_as::<_, StopInRoute>(
        r#"
        SELECT 
            s.id,
            s.name,
            ST_X(s.location::geometry) as longitude,
            ST_Y(s.location::geometry) as latitude,
            rs.stop_order,
            rs.is_optional
        FROM stops s
        JOIN route_stops rs ON s.id = rs.stop_id
        WHERE rs.route_id = $1
          AND s.is_active = true
        ORDER BY rs.stop_order
        "#,
    )
    .bind(id)
    .fetch_all(&state.db)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    let stops_count = stops.len() as i32;

    Ok(Json(RouteDetailResponse {
        route: RouteResponse::from(route),
        stops,
        stops_count,
    }))
}

/// GET /routes/:id/schedules - pełny rozkład jazdy linii
///
/// Zwraca rozkład pogrupowany według przystanków
#[derive(Debug, Serialize)]
pub struct RouteScheduleResponse {
    pub route: RouteResponse,
    pub schedules_by_stop: Vec<StopSchedules>,
}

#[derive(Debug, Serialize)]
pub struct StopSchedules {
    pub stop_id: Uuid,
    pub stop_name: String,
    pub stop_order: i32,
    pub weekday_departures: Vec<String>,
    pub saturday_departures: Vec<String>,
    pub sunday_departures: Vec<String>,
}

pub async fn get_route_schedules(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<RouteScheduleResponse>, AppError> {
    // Pobierz trasę
    let route = sqlx::query_as::<_, Route>(
        r#"
        SELECT 
            id,
            name,
            number,
            description,
            color,
            is_active,
            created_at
        FROM routes
        WHERE id = $1 AND is_active = true
        "#,
    )
    .bind(id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?
    .ok_or_else(|| AppError::NotFound(format!("Linia o ID {} nie istnieje", id)))?;

    // Pobierz przystanki z rozkładem
    let stops = sqlx::query_as::<_, StopInRoute>(
        r#"
        SELECT 
            s.id,
            s.name,
            ST_X(s.location::geometry) as longitude,
            ST_Y(s.location::geometry) as latitude,
            rs.stop_order,
            rs.is_optional
        FROM stops s
        JOIN route_stops rs ON s.id = rs.stop_id
        WHERE rs.route_id = $1
          AND s.is_active = true
        ORDER BY rs.stop_order
        "#,
    )
    .bind(id)
    .fetch_all(&state.db)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    let mut schedules_by_stop = Vec::new();

    for stop in &stops {
        // Pobierz odjazdy dla danego przystanku pogrupowane według dnia
        let weekday_departures: Vec<String> = sqlx::query_scalar(
            r#"
            SELECT departure_time::text
            FROM schedules
            WHERE route_id = $1 
              AND stop_id = $2 
              AND is_active = true
              AND day_type IN ('weekday', 'everyday')
            ORDER BY departure_time
            "#,
        )
        .bind(id)
        .bind(stop.id)
        .fetch_all(&state.db)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        let saturday_departures: Vec<String> = sqlx::query_scalar(
            r#"
            SELECT departure_time::text
            FROM schedules
            WHERE route_id = $1 
              AND stop_id = $2 
              AND is_active = true
              AND day_type IN ('saturday', 'everyday')
            ORDER BY departure_time
            "#,
        )
        .bind(id)
        .bind(stop.id)
        .fetch_all(&state.db)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        let sunday_departures: Vec<String> = sqlx::query_scalar(
            r#"
            SELECT departure_time::text
            FROM schedules
            WHERE route_id = $1 
              AND stop_id = $2 
              AND is_active = true
              AND day_type IN ('sunday', 'holiday', 'everyday')
            ORDER BY departure_time
            "#,
        )
        .bind(id)
        .bind(stop.id)
        .fetch_all(&state.db)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        schedules_by_stop.push(StopSchedules {
            stop_id: stop.id,
            stop_name: stop.name.clone(),
            stop_order: stop.stop_order,
            weekday_departures,
            saturday_departures,
            sunday_departures,
        });
    }

    Ok(Json(RouteScheduleResponse {
        route: RouteResponse::from(route),
        schedules_by_stop,
    }))
}

/// GET /routes/:id/geometry - geometria trasy dla mapy
///
/// Zwraca listę współrzędnych przystanków w kolejności trasy
#[derive(Debug, Serialize)]
pub struct RouteGeometryResponse {
    pub route_id: Uuid,
    pub route_name: String,
    pub route_color: String,
    pub coordinates: Vec<Coordinate>,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct Coordinate {
    pub lat: f64,
    pub lon: f64,
    pub stop_name: String,
    pub stop_order: i32,
}

pub async fn get_route_geometry(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<RouteGeometryResponse>, AppError> {
    let route = sqlx::query_as::<_, Route>(
        r#"
        SELECT 
            id,
            name,
            number,
            description,
            color,
            is_active,
            created_at
        FROM routes
        WHERE id = $1 AND is_active = true
        "#,
    )
    .bind(id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?
    .ok_or_else(|| AppError::NotFound(format!("Linia o ID {} nie istnieje", id)))?;

    let coordinates = sqlx::query_as::<_, Coordinate>(
        r#"
        SELECT 
            ST_Y(s.location::geometry) as lat,
            ST_X(s.location::geometry) as lon,
            s.name as stop_name,
            rs.stop_order
        FROM stops s
        JOIN route_stops rs ON s.id = rs.stop_id
        WHERE rs.route_id = $1
          AND s.is_active = true
        ORDER BY rs.stop_order
        "#,
    )
    .bind(id)
    .fetch_all(&state.db)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    Ok(Json(RouteGeometryResponse {
        route_id: route.id,
        route_name: route.name,
        route_color: route.color,
        coordinates,
    }))
}

/// GET /routes/search?query={} - wyszukiwanie linii
#[derive(Debug, Deserialize)]
pub struct SearchRoutesQuery {
    pub query: String,
}

pub async fn search_routes(
    State(state): State<AppState>,
    Query(query): Query<SearchRoutesQuery>,
) -> Result<Json<Vec<RouteResponse>>, AppError> {
    if query.query.trim().is_empty() {
        return Err(AppError::ValidationError(
            "Zapytanie wyszukiwania nie może być puste".to_string(),
        ));
    }

    let search_pattern = format!("%{}%", query.query);

    let routes = sqlx::query_as::<_, Route>(
        r#"
        SELECT 
            id,
            name,
            number,
            description,
            color,
            is_active,
            created_at
        FROM routes
        WHERE is_active = true
          AND (
              number ILIKE $1
              OR name ILIKE $1
              OR description ILIKE $1
          )
        ORDER BY 
            CASE 
                WHEN number ~ '^[0-9]+$' THEN number::int
                ELSE 999999
            END,
            number
        LIMIT 20
        "#,
    )
    .bind(&search_pattern)
    .fetch_all(&state.db)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    let route_responses: Vec<RouteResponse> = routes.into_iter().map(RouteResponse::from).collect();

    Ok(Json(route_responses))
}
