//! Handlers for trip execution and stop-event log (Phase 2 — issue #26).
//!
//! Endpoints:
//! - POST   /trips              – create a trip execution record
//! - GET    /trips              – list trips (filter: vehicle_id, route_id, trip_date, status)
//! - GET    /trips/:id          – trip detail with full event log
//! - PATCH  /trips/:id          – update trip status / timestamps
//! - POST   /trips/:id/events   – record a stop arrival/departure
//! - GET    /trips/:id/events   – list events for a trip

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use chrono::{Days, Duration, Utc};
use uuid::Uuid;

use crate::{
    errors::AppError,
    models::trip::{
        CreateTripEventRequest, CreateTripRequest, Trip, TripDetailResponse, TripEvent, TripsQuery,
        UpdateTripRequest,
    },
    state::AppState,
};

// ── POST /trips ───────────────────────────────────────────────────────────────

pub async fn create_trip(
    State(state): State<AppState>,
    Json(req): Json<CreateTripRequest>,
) -> Result<(StatusCode, Json<Trip>), AppError> {
    let retain_until = req.retain_until.unwrap_or_else(|| {
        req.trip_date
            .checked_add_days(Days::new(30))
            .unwrap_or(req.trip_date)
    });

    if retain_until < req.trip_date {
        return Err(AppError::ValidationError(
            "retain_until must not be before trip_date".to_string(),
        ));
    }

    let trip = sqlx::query_as::<_, Trip>(
        r#"
        INSERT INTO trips (vehicle_id, route_id, driver_id, trip_date, retain_until)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING *
        "#,
    )
    .bind(req.vehicle_id)
    .bind(req.route_id)
    .bind(req.driver_id)
    .bind(req.trip_date)
    .bind(retain_until)
    .fetch_one(&state.db)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    Ok((StatusCode::CREATED, Json(trip)))
}

// ── GET /trips ────────────────────────────────────────────────────────────────

pub async fn list_trips(
    State(state): State<AppState>,
    Query(query): Query<TripsQuery>,
) -> Result<Json<Vec<Trip>>, AppError> {
    use sqlx::QueryBuilder;
    let mut qb = QueryBuilder::<sqlx::Postgres>::new("SELECT * FROM trips WHERE 1=1");
    if let Some(v) = query.vehicle_id {
        qb.push(" AND vehicle_id = ").push_bind(v);
    }
    if let Some(r) = query.route_id {
        qb.push(" AND route_id = ").push_bind(r);
    }
    if let Some(d) = query.trip_date {
        qb.push(" AND trip_date = ").push_bind(d);
    }
    if let Some(s) = query.status {
        qb.push(" AND status = ").push_bind(s);
    }
    qb.push(" ORDER BY trip_date DESC, created_at DESC LIMIT 200");

    let trips = qb
        .build_query_as::<Trip>()
        .fetch_all(&state.db)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    Ok(Json(trips))
}

// ── GET /trips/:id ────────────────────────────────────────────────────────────

pub async fn get_trip(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<TripDetailResponse>, AppError> {
    let trip = sqlx::query_as::<_, Trip>("SELECT * FROM trips WHERE id = $1")
        .bind(id)
        .fetch_optional(&state.db)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?
        .ok_or_else(|| AppError::NotFound(format!("Trip {} not found", id)))?;

    let events = sqlx::query_as::<_, TripEvent>(
        "SELECT * FROM trip_events WHERE trip_id = $1 ORDER BY occurred_at ASC",
    )
    .bind(id)
    .fetch_all(&state.db)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    Ok(Json(TripDetailResponse { trip, events }))
}

// ── PATCH /trips/:id ──────────────────────────────────────────────────────────

pub async fn update_trip(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateTripRequest>,
) -> Result<Json<Trip>, AppError> {
    // Verify existence
    let _exists = sqlx::query_scalar::<_, bool>("SELECT EXISTS(SELECT 1 FROM trips WHERE id = $1)")
        .bind(id)
        .fetch_one(&state.db)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    if !_exists {
        return Err(AppError::NotFound(format!("Trip {} not found", id)));
    }

    use sqlx::QueryBuilder;
    let mut qb = QueryBuilder::<sqlx::Postgres>::new("UPDATE trips SET updated_at = NOW()");

    if let Some(s) = req.status {
        qb.push(", status = ").push_bind(s);
    }
    if let Some(ts) = req.started_at {
        qb.push(", started_at = ").push_bind(ts);
    }
    if let Some(ts) = req.ended_at {
        qb.push(", ended_at = ").push_bind(ts);
    }

    qb.push(" WHERE id = ").push_bind(id).push(" RETURNING *");

    let trip = qb
        .build_query_as::<Trip>()
        .fetch_one(&state.db)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    Ok(Json(trip))
}

// ── POST /trips/:id/events ────────────────────────────────────────────────────

pub async fn create_trip_event(
    State(state): State<AppState>,
    Path(trip_id): Path<Uuid>,
    Json(req): Json<CreateTripEventRequest>,
) -> Result<(StatusCode, Json<TripEvent>), AppError> {
    // Trip must exist
    let trip_exists =
        sqlx::query_scalar::<_, bool>("SELECT EXISTS(SELECT 1 FROM trips WHERE id = $1)")
            .bind(trip_id)
            .fetch_one(&state.db)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    if !trip_exists {
        return Err(AppError::NotFound(format!("Trip {} not found", trip_id)));
    }

    // Validate: both lat/lon present or both absent
    match (req.latitude, req.longitude) {
        (Some(_), None) | (None, Some(_)) => {
            return Err(AppError::ValidationError(
                "latitude and longitude must both be provided or both omitted".to_string(),
            ));
        }
        _ => {}
    }

    // Validate: occurred_at not in the future (60-second grace)
    let grace = Utc::now() + Duration::seconds(60);
    if req.occurred_at > grace {
        return Err(AppError::ValidationError(
            "occurred_at must not be in the future".to_string(),
        ));
    }

    let event = sqlx::query_as::<_, TripEvent>(
        r#"
        INSERT INTO trip_events
            (trip_id, stop_id, schedule_id, event_type, occurred_at,
             latitude, longitude, delay_seconds)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        RETURNING *
        "#,
    )
    .bind(trip_id)
    .bind(req.stop_id)
    .bind(req.schedule_id)
    .bind(req.event_type)
    .bind(req.occurred_at)
    .bind(req.latitude)
    .bind(req.longitude)
    .bind(req.delay_seconds)
    .fetch_one(&state.db)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    Ok((StatusCode::CREATED, Json(event)))
}

// ── GET /trips/:id/events ─────────────────────────────────────────────────────

pub async fn list_trip_events(
    State(state): State<AppState>,
    Path(trip_id): Path<Uuid>,
) -> Result<Json<Vec<TripEvent>>, AppError> {
    let trip_exists =
        sqlx::query_scalar::<_, bool>("SELECT EXISTS(SELECT 1 FROM trips WHERE id = $1)")
            .bind(trip_id)
            .fetch_one(&state.db)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    if !trip_exists {
        return Err(AppError::NotFound(format!("Trip {} not found", trip_id)));
    }

    let events = sqlx::query_as::<_, TripEvent>(
        "SELECT * FROM trip_events WHERE trip_id = $1 ORDER BY occurred_at ASC",
    )
    .bind(trip_id)
    .fetch_all(&state.db)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    Ok(Json(events))
}
