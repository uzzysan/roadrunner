//! Handler dla pojazdów (vehicles) z obsługą GPS
//!
//! Endpointy:
//! - GET /vehicles - lista pojazdów
//! - POST /vehicles - dodaj pojazd
//! - GET /vehicles/:id - szczegóły pojazdu
//! - PUT /vehicles/:id - edytuj pojazd
//! - DELETE /vehicles/:id - usuń pojazd
//! - POST /vehicles/:id/location - aktualizuj lokalizację (z GPS)
//! - GET /vehicles/:id/history - historia lokalizacji
//! - POST /vehicles/:id/assign-driver - przypisz kierowcę
//! - POST /vehicles/:id/assign-route - przypisz trasę
//! - GET /vehicles/active - aktywne pojazdy na trasach
//! - GET /vehicles/available - dostępne pojazdy (nieprzypisane)

use axum::{
    extract::{Path, Query, State},
    Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    models::vehicle::{
        Vehicle, VehicleResponse, VehicleLocationDetail, CreateVehicleRequest,
        UpdateVehicleRequest, UpdateVehicleLocationRequest, AssignDriverRequest,
        AssignRouteRequest, make_point, VehicleStatus,
    },
    state::AppState,
    errors::AppError,
};

/// Query parameters dla filtrowania pojazdów
#[derive(Debug, Deserialize)]
pub struct VehiclesQuery {
    /// Filtrowanie po statusie
    pub status: Option<VehicleStatus>,
    /// Filtrowanie po typie
    pub vehicle_type: Option<String>,
    /// Filtrowanie po trasie
    pub route_id: Option<Uuid>,
    /// Filtrowanie po kierowcy
    pub driver_id: Option<Uuid>,
}

/// Response dla listy pojazdów
#[derive(Debug, Serialize)]
pub struct VehiclesListResponse {
    pub vehicles: Vec<VehicleResponse>,
    pub total: i64,
}

/// GET /vehicles - lista wszystkich pojazdów
pub async fn list_vehicles(
    State(state): State<AppState>,
    Query(query): Query<VehiclesQuery>,
) -> Result<Json<VehiclesListResponse>, AppError> {
    let mut sql = String::from(
        r#"
        SELECT 
            id, registration_number, vin, brand, model, year,
            capacity, vehicle_type as "vehicle_type: _",
            fuel_type as "fuel_type: _",
            status as "status: _",
            gps_device_id, ST_AsText(last_location) as last_location,
            last_location_at, current_driver_id, current_route_id,
            created_at, updated_at
        FROM vehicles
        WHERE 1=1
        "#
    );

    if let Some(status) = query.status {
        sql.push_str(&format!(" AND status = '{:?}'", status).to_lowercase());
    }

    if let Some(ref vtype) = query.vehicle_type {
        sql.push_str(&format!(" AND vehicle_type = '{}'", vtype));
    }

    if let Some(route_id) = query.route_id {
        sql.push_str(&format!(" AND current_route_id = '{}'", route_id));
    }

    if let Some(driver_id) = query.driver_id {
        sql.push_str(&format!(" AND current_driver_id = '{}'", driver_id));
    }

    sql.push_str(" ORDER BY brand, model, registration_number");

    let vehicles = sqlx::query_as::<_, Vehicle>(&sql)
        .fetch_all(&state.db)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    let total = vehicles.len() as i64;
    let vehicle_responses: Vec<VehicleResponse> = vehicles.into_iter()
        .map(VehicleResponse::from)
        .collect();

    Ok(Json(VehiclesListResponse {
        vehicles: vehicle_responses,
        total,
    }))
}

/// POST /vehicles - dodaj nowy pojazd
pub async fn create_vehicle(
    State(state): State<AppState>,
    Json(request): Json<CreateVehicleRequest>,
) -> Result<Json<VehicleResponse>, AppError> {
    // Walidacja
    if request.registration_number.trim().is_empty() {
        return Err(AppError::ValidationError(
            "Numer rejestracyjny jest wymagany".to_string()
        ));
    }

    if request.brand.trim().is_empty() || request.model.trim().is_empty() {
        return Err(AppError::ValidationError(
            "Marka i model są wymagane".to_string()
        ));
    }

    if request.capacity <= 0 {
        return Err(AppError::ValidationError(
            "Pojemność musi być większa od 0".to_string()
        ));
    }

    // Sprawdź czy rejestracja już istnieje
    let exists: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM vehicles WHERE registration_number = $1)"
    )
    .bind(&request.registration_number)
    .fetch_one(&state.db)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    if exists {
        return Err(AppError::ValidationError(
            "Pojazd o tym numerze rejestracyjnym już istnieje".to_string()
        ));
    }

    let vehicle = sqlx::query_as::<_, Vehicle>(
        r#"
        INSERT INTO vehicles (
            registration_number, vin, brand, model, year, capacity,
            vehicle_type, fuel_type, gps_device_id
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
        RETURNING 
            id, registration_number, vin, brand, model, year,
            capacity, vehicle_type, fuel_type, status,
            gps_device_id, ST_AsText(last_location) as last_location,
            last_location_at, current_driver_id, current_route_id,
            created_at, updated_at
        "#
    )
    .bind(&request.registration_number)
    .bind(&request.vin)
    .bind(&request.brand)
    .bind(&request.model)
    .bind(request.year)
    .bind(request.capacity)
    .bind(request.vehicle_type)
    .bind(request.fuel_type)
    .bind(&request.gps_device_id)
    .fetch_one(&state.db)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    Ok(Json(VehicleResponse::from(vehicle)))
}

/// GET /vehicles/:id - szczegóły pojazdu
pub async fn get_vehicle(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<VehicleResponse>, AppError> {
    let vehicle = sqlx::query_as::<_, Vehicle>(
        r#"
        SELECT 
            id, registration_number, vin, brand, model, year,
            capacity, vehicle_type as "vehicle_type: _",
            fuel_type as "fuel_type: _",
            status as "status: _",
            gps_device_id, ST_AsText(last_location) as last_location,
            last_location_at, current_driver_id, current_route_id,
            created_at, updated_at
        FROM vehicles
        WHERE id = $1
        "#
    )
    .bind(id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?
    .ok_or_else(|| AppError::NotFound(format!("Pojazd o ID {} nie istnieje", id)))?;

    Ok(Json(VehicleResponse::from(vehicle)))
}

/// PUT /vehicles/:id - edytuj pojazd
pub async fn update_vehicle(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(request): Json<UpdateVehicleRequest>,
) -> Result<Json<VehicleResponse>, AppError> {
    // Sprawdź czy pojazd istnieje
    let exists: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM vehicles WHERE id = $1)"
    )
    .bind(id)
    .fetch_one(&state.db)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    if !exists {
        return Err(AppError::NotFound(format!("Pojazd o ID {} nie istnieje", id)));
    }

    // Sprawdź unikalność rejestracji jeśli zmieniana
    if let Some(ref reg) = request.registration_number {
        let reg_exists: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM vehicles WHERE registration_number = $1 AND id != $2)"
        )
        .bind(reg)
        .bind(id)
        .fetch_one(&state.db)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        if reg_exists {
            return Err(AppError::ValidationError(
                "Pojazd o tym numerze rejestracyjnym już istnieje".to_string()
            ));
        }
    }

    let vehicle = sqlx::query_as::<_, Vehicle>(
        r#"
        UPDATE vehicles
        SET 
            registration_number = COALESCE($1, registration_number),
            vin = COALESCE($2, vin),
            brand = COALESCE($3, brand),
            model = COALESCE($4, model),
            year = COALESCE($5, year),
            capacity = COALESCE($6, capacity),
            vehicle_type = COALESCE($7, vehicle_type),
            fuel_type = COALESCE($8, fuel_type),
            status = COALESCE($9, status),
            gps_device_id = COALESCE($10, gps_device_id),
            updated_at = NOW()
        WHERE id = $11
        RETURNING 
            id, registration_number, vin, brand, model, year,
            capacity, vehicle_type as "vehicle_type: _",
            fuel_type as "fuel_type: _",
            status as "status: _",
            gps_device_id, ST_AsText(last_location) as last_location,
            last_location_at, current_driver_id, current_route_id,
            created_at, updated_at
        "#
    )
    .bind(&request.registration_number)
    .bind(&request.vin)
    .bind(&request.brand)
    .bind(&request.model)
    .bind(request.year)
    .bind(request.capacity)
    .bind(request.vehicle_type)
    .bind(request.fuel_type)
    .bind(request.status)
    .bind(&request.gps_device_id)
    .bind(id)
    .fetch_one(&state.db)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    Ok(Json(VehicleResponse::from(vehicle)))
}

/// DELETE /vehicles/:id - usuń pojazd
pub async fn delete_vehicle(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    let result = sqlx::query("DELETE FROM vehicles WHERE id = $1")
        .bind(id)
        .execute(&state.db)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound(format!("Pojazd o ID {} nie istnieje", id)));
    }

    Ok(Json(serde_json::json!({
        "message": "Pojazd został usunięty",
        "vehicle_id": id
    })))
}

/// POST /vehicles/:id/location - aktualizuj lokalizację (z GPS)
pub async fn update_vehicle_location(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(request): Json<UpdateVehicleLocationRequest>,
) -> Result<Json<VehicleResponse>, AppError> {
    // Walidacja współrzędnych
    if request.latitude < -90.0 || request.latitude > 90.0 {
        return Err(AppError::ValidationError(
            "Nieprawidłowa szerokość geograficzna".to_string()
        ));
    }
    if request.longitude < -180.0 || request.longitude > 180.0 {
        return Err(AppError::ValidationError(
            "Nieprawidłowa długość geograficzna".to_string()
        ));
    }

    let location_wkt = make_point(request.longitude, request.latitude);

    let vehicle = sqlx::query_as::<_, Vehicle>(
        r#"
        UPDATE vehicles
        SET 
            last_location = ST_GeogFromText($1),
            last_location_at = NOW(),
            updated_at = NOW()
        WHERE id = $2
        RETURNING 
            id, registration_number, vin, brand, model, year,
            capacity, vehicle_type as "vehicle_type: _",
            fuel_type as "fuel_type: _",
            status as "status: _",
            gps_device_id, ST_AsText(last_location) as last_location,
            last_location_at, current_driver_id, current_route_id,
            created_at, updated_at
        "#
    )
    .bind(&location_wkt)
    .bind(id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?
    .ok_or_else(|| AppError::NotFound(format!("Pojazd o ID {} nie istnieje", id)))?;

    // Opcjonalnie: zapisz historię lokalizacji
    if let Err(e) = sqlx::query(
        r#"
        INSERT INTO vehicle_locations (vehicle_id, location, speed, heading, next_stop_id, eta_seconds)
        VALUES ($1, ST_GeogFromText($2), $3, $4, $5, $6)
        "#
    )
    .bind(id)
    .bind(&location_wkt)
    .bind(request.speed)
    .bind(request.heading)
    .bind(request.next_stop_id)
    .bind(request.eta_seconds)
    .execute(&state.db)
    .await {
        tracing::warn!("Failed to save location history: {}", e);
    }

    Ok(Json(VehicleResponse::from(vehicle)))
}

/// GET /vehicles/:id/history - historia lokalizacji
#[derive(Debug, Serialize)]
pub struct LocationHistoryResponse {
    pub locations: Vec<LocationHistoryItem>,
}

#[derive(Debug, Serialize)]
pub struct LocationHistoryItem {
    pub latitude: f64,
    pub longitude: f64,
    pub speed: Option<f64>,
    pub heading: Option<f64>,
    pub recorded_at: DateTime<Utc>,
}

pub async fn get_vehicle_location_history(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Query(query): Query<LocationHistoryQuery>,
) -> Result<Json<LocationHistoryResponse>, AppError> {
    let limit = query.limit.unwrap_or(100);

    let rows = sqlx::query(
        r#"
        SELECT 
            ST_X(location::geometry) as longitude,
            ST_Y(location::geometry) as latitude,
            speed, heading, recorded_at
        FROM vehicle_locations
        WHERE vehicle_id = $1
        ORDER BY recorded_at DESC
        LIMIT $2
        "#
    )
    .bind(id)
    .bind(limit)
    .fetch_all(&state.db)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    let locations: Vec<LocationHistoryItem> = rows.into_iter()
        .map(|row| LocationHistoryItem {
            longitude: row.get("longitude"),
            latitude: row.get("latitude"),
            speed: row.get("speed"),
            heading: row.get("heading"),
            recorded_at: row.get("recorded_at"),
        })
        .collect();

    Ok(Json(LocationHistoryResponse { locations }))
}

#[derive(Debug, Deserialize)]
pub struct LocationHistoryQuery {
    pub limit: Option<i32>,
}

/// POST /vehicles/:id/assign-driver - przypisz kierowcę
pub async fn assign_driver(
    State(state): State<AppState>,
    Path(vehicle_id): Path<Uuid>,
    Json(request): Json<AssignDriverRequest>,
) -> Result<Json<VehicleResponse>, AppError> {
    // Użyj funkcji SQL do przypisania
    let success: bool = sqlx::query_scalar(
        "SELECT assign_driver_to_vehicle($1, $2)"
    )
    .bind(request.driver_id)
    .bind(vehicle_id)
    .fetch_one(&state.db)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    if !success {
        return Err(AppError::InternalError(
            "Nie udało się przypisać kierowcy".to_string()
        ));
    }

    // Pobierz zaktualizowany pojazd
    let vehicle = sqlx::query_as::<_, Vehicle>(
        r#"
        SELECT 
            id, registration_number, vin, brand, model, year,
            capacity, vehicle_type as "vehicle_type: _",
            fuel_type as "fuel_type: _",
            status as "status: _",
            gps_device_id, ST_AsText(last_location) as last_location,
            last_location_at, current_driver_id, current_route_id,
            created_at, updated_at
        FROM vehicles
        WHERE id = $1
        "#
    )
    .bind(vehicle_id)
    .fetch_one(&state.db)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    Ok(Json(VehicleResponse::from(vehicle)))
}

/// POST /vehicles/:id/assign-route - przypisz trasę
pub async fn assign_route(
    State(state): State<AppState>,
    Path(vehicle_id): Path<Uuid>,
    Json(request): Json<AssignRouteRequest>,
) -> Result<Json<VehicleResponse>, AppError> {
    // Sprawdź czy trasa istnieje
    let route_exists: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM routes WHERE id = $1 AND is_active = true)"
    )
    .bind(request.route_id)
    .fetch_one(&state.db)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    if !route_exists {
        return Err(AppError::NotFound("Trasa nie istnieje lub jest nieaktywna".to_string()));
    }

    let vehicle = sqlx::query_as::<_, Vehicle>(
        r#"
        UPDATE vehicles
        SET current_route_id = $1, updated_at = NOW()
        WHERE id = $2
        RETURNING 
            id, registration_number, vin, brand, model, year,
            capacity, vehicle_type as "vehicle_type: _",
            fuel_type as "fuel_type: _",
            status as "status: _",
            gps_device_id, ST_AsText(last_location) as last_location,
            last_location_at, current_driver_id, current_route_id,
            created_at, updated_at
        "#
    )
    .bind(request.route_id)
    .bind(vehicle_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?
    .ok_or_else(|| AppError::NotFound(format!("Pojazd o ID {} nie istnieje", vehicle_id)))?;

    Ok(Json(VehicleResponse::from(vehicle)))
}

/// GET /vehicles/active - aktywne pojazdy na trasach
pub async fn get_active_vehicles(
    State(state): State<AppState>,
) -> Result<Json<Vec<VehicleLocationDetail>>, AppError> {
    let vehicles = sqlx::query_as::<_, VehicleLocationDetail>(
        r#"
        SELECT 
            v.id as vehicle_id,
            v.registration_number,
            r.id as route_id,
            r.name as route_name,
            r.number as route_number,
            r.color as route_color,
            d.id as driver_id,
            CONCAT(u.first_name, ' ', u.last_name) as driver_name,
            ST_Y(v.last_location::geometry) as latitude,
            ST_X(v.last_location::geometry) as longitude,
            NULL::double precision as speed,
            NULL::double precision as heading,
            NULL::uuid as next_stop_id,
            NULL::varchar as next_stop_name,
            NULL::integer as eta_seconds,
            v.last_location_at as recorded_at
        FROM vehicles v
        JOIN routes r ON v.current_route_id = r.id
        LEFT JOIN drivers d ON v.current_driver_id = d.id
        LEFT JOIN users u ON d.user_id = u.id
        WHERE v.status = 'active'
          AND v.current_route_id IS NOT NULL
          AND v.last_location IS NOT NULL
        ORDER BY r.number, v.registration_number
        "#
    )
    .fetch_all(&state.db)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    Ok(Json(vehicles))
}

/// GET /vehicles/available - dostępne pojazdy (nieprzypisane do trasy)
pub async fn get_available_vehicles(
    State(state): State<AppState>,
) -> Result<Json<VehiclesListResponse>, AppError> {
    let vehicles = sqlx::query_as::<_, Vehicle>(
        r#"
        SELECT 
            id, registration_number, vin, brand, model, year,
            capacity, vehicle_type as "vehicle_type: _",
            fuel_type as "fuel_type: _",
            status as "status: _",
            gps_device_id, ST_AsText(last_location) as last_location,
            last_location_at, current_driver_id, current_route_id,
            created_at, updated_at
        FROM vehicles
        WHERE status = 'active'
          AND current_route_id IS NULL
        ORDER BY brand, model, registration_number
        "#
    )
    .fetch_all(&state.db)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    let total = vehicles.len() as i64;
    let vehicle_responses: Vec<VehicleResponse> = vehicles.into_iter()
        .map(VehicleResponse::from)
        .collect();

    Ok(Json(VehiclesListResponse {
        vehicles: vehicle_responses,
        total,
    }))
}
