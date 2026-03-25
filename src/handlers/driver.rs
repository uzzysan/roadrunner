//! Handler dla kierowców (drivers)
//!
//! Endpointy:
//! - GET /drivers - lista kierowców
//! - POST /drivers - dodaj kierowcę
//! - GET /drivers/:id - szczegóły kierowcy
//! - PUT /drivers/:id - edytuj kierowcę
//! - DELETE /drivers/:id - usuń kierowcę
//! - POST /drivers/:id/assign-vehicle - przypisz pojazd
//! - POST /drivers/:id/unassign - usuń przypisanie
//! - GET /drivers/:id/dashboard - dashboard kierowcy (mobile)
//! - GET /drivers/available - dostępni kierowcy

use axum::{
    extract::{Path, Query, State},
    Json,
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    models::driver::{
        Driver, DriverResponse, DriverWithUser, DriverDashboardInfo,
        CreateDriverRequest, UpdateDriverRequest, AssignVehicleRequest,
        DriverStatus, AssignedVehicleInfo, DriverRouteInfo, is_license_valid,
    },
    state::AppState,
    errors::AppError,
};

/// Query parameters dla filtrowania kierowców
#[derive(Debug, Deserialize)]
pub struct DriversQuery {
    pub status: Option<DriverStatus>,
    pub has_vehicle: Option<bool>,
}

/// Response dla listy kierowców
#[derive(Debug, Serialize)]
pub struct DriversListResponse {
    pub drivers: Vec<DriverWithUser>,
    pub total: i64,
}

/// GET /drivers - lista wszystkich kierowców
pub async fn list_drivers(
    State(state): State<AppState>,
    Query(query): Query<DriversQuery>,
) -> Result<Json<DriversListResponse>, AppError> {
    let mut sql = String::from(
        r#"
        SELECT 
            d.id, d.user_id, d.employee_id, d.license_number,
            d.license_categories, d.license_expiry, d.phone,
            d.emergency_contact, d.status as "status: _",
            d.assigned_vehicle_id, d.created_at, d.updated_at,
            u.first_name, u.last_name, u.email
        FROM drivers d
        JOIN users u ON d.user_id = u.id
        WHERE 1=1
        "#
    );

    if let Some(status) = query.status {
        sql.push_str(&format!(" AND d.status = '{:?}'", status).to_lowercase());
    }

    if let Some(has_vehicle) = query.has_vehicle {
        if has_vehicle {
            sql.push_str(" AND d.assigned_vehicle_id IS NOT NULL");
        } else {
            sql.push_str(" AND d.assigned_vehicle_id IS NULL");
        }
    }

    sql.push_str(" ORDER BY u.last_name, u.first_name");

    let rows = sqlx::query(&sql)
        .fetch_all(&state.db)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    let mut drivers = Vec::new();

    for row in rows {
        let driver = Driver {
            id: row.get("id"),
            user_id: row.get("user_id"),
            employee_id: row.get("employee_id"),
            license_number: row.get("license_number"),
            license_categories: row.get("license_categories"),
            license_expiry: row.get("license_expiry"),
            phone: row.get("phone"),
            emergency_contact: row.get("emergency_contact"),
            status: row.get("status"),
            assigned_vehicle_id: row.get("assigned_vehicle_id"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        };

        let mut driver_response = DriverResponse::from(driver);

        // Pobierz info o pojeździe jeśli przypisany
        if let Some(vehicle_id) = driver_response.assigned_vehicle_id {
            if let Ok(vehicle) = sqlx::query(
                "SELECT id, registration_number, brand, model FROM vehicles WHERE id = $1"
            )
            .bind(vehicle_id)
            .fetch_one(&state.db)
            .await {
                driver_response.assigned_vehicle_info = Some(AssignedVehicleInfo {
                    id: vehicle.get("id"),
                    registration_number: vehicle.get("registration_number"),
                    brand: vehicle.get("brand"),
                    model: vehicle.get("model"),
                });
            }
        }

        let driver_with_user = DriverWithUser {
            driver: driver_response,
            first_name: row.get("first_name"),
            last_name: row.get("last_name"),
            email: row.get("email"),
            full_name: format!("{} {}", 
                row.get::<String, _>("first_name"), 
                row.get::<String, _>("last_name")
            ),
        };

        drivers.push(driver_with_user);
    }

    let total = drivers.len() as i64;

    Ok(Json(DriversListResponse {
        drivers,
        total,
    }))
}

/// POST /drivers - dodaj nowego kierowcę
pub async fn create_driver(
    State(state): State<AppState>,
    Json(request): Json<CreateDriverRequest>,
) -> Result<Json<DriverResponse>, AppError> {
    // Walidacja
    if request.license_number.trim().is_empty() {
        return Err(AppError::ValidationError(
            "Numer prawa jazdy jest wymagany".to_string()
        ));
    }

    if request.license_categories.is_empty() {
        return Err(AppError::ValidationError(
            "Co najmniej jedna kategoria prawa jazdy jest wymagana".to_string()
        ));
    }

    if !is_license_valid(request.license_expiry) {
        return Err(AppError::ValidationError(
            "Prawo jazdy jest nieważne lub wygasło".to_string()
        ));
    }

    // Sprawdź czy użytkownik istnieje
    let user_exists: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM users WHERE id = $1)"
    )
    .bind(request.user_id)
    .fetch_one(&state.db)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    if !user_exists {
        return Err(AppError::NotFound("Użytkownik nie istnieje".to_string()));
    }

    // Sprawdź czy użytkownik już nie jest kierowcą
    let driver_exists: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM drivers WHERE user_id = $1)"
    )
    .bind(request.user_id)
    .fetch_one(&state.db)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    if driver_exists {
        return Err(AppError::ValidationError(
            "Ten użytkownik jest już zarejestrowany jako kierowca".to_string()
        ));
    }

    let driver = sqlx::query_as::<_, Driver>(
        r#"
        INSERT INTO drivers (
            user_id, employee_id, license_number, license_categories,
            license_expiry, phone, emergency_contact
        ) VALUES ($1, $2, $3, $4, $5, $6, $7)
        RETURNING 
            id, user_id, employee_id, license_number, license_categories,
            license_expiry, phone, emergency_contact, status as "status: _",
            assigned_vehicle_id, created_at, updated_at
        "#
    )
    .bind(request.user_id)
    .bind(&request.employee_id)
    .bind(&request.license_number)
    .bind(&request.license_categories)
    .bind(request.license_expiry)
    .bind(&request.phone)
    .bind(&request.emergency_contact)
    .fetch_one(&state.db)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    Ok(Json(DriverResponse::from(driver)))
}

/// GET /drivers/:id - szczegóły kierowcy
pub async fn get_driver(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<DriverWithUser>, AppError> {
    let row = sqlx::query(
        r#"
        SELECT 
            d.id, d.user_id, d.employee_id, d.license_number,
            d.license_categories, d.license_expiry, d.phone,
            d.emergency_contact, d.status as "status: _",
            d.assigned_vehicle_id, d.created_at, d.updated_at,
            u.first_name, u.last_name, u.email
        FROM drivers d
        JOIN users u ON d.user_id = u.id
        WHERE d.id = $1
        "#
    )
    .bind(id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?
    .ok_or_else(|| AppError::NotFound(format!("Kierowca o ID {} nie istnieje", id)))?;

    let driver = Driver {
        id: row.get("id"),
        user_id: row.get("user_id"),
        employee_id: row.get("employee_id"),
        license_number: row.get("license_number"),
        license_categories: row.get("license_categories"),
        license_expiry: row.get("license_expiry"),
        phone: row.get("phone"),
        emergency_contact: row.get("emergency_contact"),
        status: row.get("status"),
        assigned_vehicle_id: row.get("assigned_vehicle_id"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    };

    let mut driver_response = DriverResponse::from(driver);

    // Pobierz info o pojeździe
    if let Some(vehicle_id) = driver_response.assigned_vehicle_id {
        if let Ok(vehicle) = sqlx::query(
            "SELECT id, registration_number, brand, model FROM vehicles WHERE id = $1"
        )
        .bind(vehicle_id)
        .fetch_one(&state.db)
        .await {
            driver_response.assigned_vehicle_info = Some(AssignedVehicleInfo {
                id: vehicle.get("id"),
                registration_number: vehicle.get("registration_number"),
                brand: vehicle.get("brand"),
                model: vehicle.get("model"),
            });
        }
    }

    let driver_with_user = DriverWithUser {
        driver: driver_response,
        first_name: row.get("first_name"),
        last_name: row.get("last_name"),
        email: row.get("email"),
        full_name: format!("{} {}", 
            row.get::<String, _>("first_name"), 
            row.get::<String, _>("last_name")
        ),
    };

    Ok(Json(driver_with_user))
}

/// PUT /drivers/:id - edytuj kierowcę
pub async fn update_driver(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(request): Json<UpdateDriverRequest>,
) -> Result<Json<DriverResponse>, AppError> {
    // Sprawdź czy kierowca istnieje
    let exists: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM drivers WHERE id = $1)"
    )
    .bind(id)
    .fetch_one(&state.db)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    if !exists {
        return Err(AppError::NotFound(format!("Kierowca o ID {} nie istnieje", id)));
    }

    // Sprawdź ważność prawa jazdy jeśli aktualizowane
    if let Some(expiry) = request.license_expiry {
        if !is_license_valid(expiry) {
            return Err(AppError::ValidationError(
                "Prawo jazdy jest nieważne lub wygasło".to_string()
            ));
        }
    }

    let driver = sqlx::query_as::<_, Driver>(
        r#"
        UPDATE drivers
        SET 
            employee_id = COALESCE($1, employee_id),
            license_number = COALESCE($2, license_number),
            license_categories = COALESCE($3, license_categories),
            license_expiry = COALESCE($4, license_expiry),
            phone = COALESCE($5, phone),
            emergency_contact = COALESCE($6, emergency_contact),
            status = COALESCE($7, status),
            updated_at = NOW()
        WHERE id = $8
        RETURNING 
            id, user_id, employee_id, license_number, license_categories,
            license_expiry, phone, emergency_contact, status as "status: _",
            assigned_vehicle_id, created_at, updated_at
        "#
    )
    .bind(&request.employee_id)
    .bind(&request.license_number)
    .bind(&request.license_categories)
    .bind(request.license_expiry)
    .bind(&request.phone)
    .bind(&request.emergency_contact)
    .bind(request.status)
    .bind(id)
    .fetch_one(&state.db)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    Ok(Json(DriverResponse::from(driver)))
}

/// DELETE /drivers/:id - usuń kierowcę
pub async fn delete_driver(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    // Sprawdź czy kierowca ma przypisany pojazd
    let has_vehicle: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM drivers WHERE id = $1 AND assigned_vehicle_id IS NOT NULL)"
    )
    .bind(id)
    .fetch_one(&state.db)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    if has_vehicle {
        return Err(AppError::ValidationError(
            "Nie można usunąć kierowcy z przypisanym pojazdem. Najpierw usuń przypisanie.".to_string()
        ));
    }

    let result = sqlx::query("DELETE FROM drivers WHERE id = $1")
        .bind(id)
        .execute(&state.db)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound(format!("Kierowca o ID {} nie istnieje", id)));
    }

    Ok(Json(serde_json::json!({
        "message": "Kierowca został usunięty",
        "driver_id": id
    })))
}

/// POST /drivers/:id/assign-vehicle - przypisz pojazd
pub async fn assign_vehicle(
    State(state): State<AppState>,
    Path(driver_id): Path<Uuid>,
    Json(request): Json<AssignVehicleRequest>,
) -> Result<Json<DriverResponse>, AppError> {
    // Użyj funkcji SQL
    let success: bool = sqlx::query_scalar(
        "SELECT assign_driver_to_vehicle($1, $2)"
    )
    .bind(driver_id)
    .bind(request.vehicle_id)
    .fetch_one(&state.db)
    .await
    .map_err(|e| {
        if e.to_string().contains("Vehicle is not active") {
            AppError::ValidationError("Pojazd nie jest aktywny".to_string())
        } else if e.to_string().contains("Driver is not active") {
            AppError::ValidationError("Kierowca nie jest aktywny".to_string())
        } else if e.to_string().contains("Driver license is expired") {
            AppError::ValidationError("Prawo jazdy kierowcy wygasło".to_string())
        } else {
            AppError::DatabaseError(e.to_string())
        }
    })?;

    if !success {
        return Err(AppError::InternalError(
            "Nie udało się przypisać pojazdu".to_string()
        ));
    }

    // Pobierz zaktualizowanego kierowcę
    let driver = sqlx::query_as::<_, Driver>(
        r#"
        SELECT 
            id, user_id, employee_id, license_number, license_categories,
            license_expiry, phone, emergency_contact, status as "status: _",
            assigned_vehicle_id, created_at, updated_at
        FROM drivers
        WHERE id = $1
        "#
    )
    .bind(driver_id)
    .fetch_one(&state.db)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    Ok(Json(DriverResponse::from(driver)))
}

/// POST /drivers/:id/unassign - usuń przypisanie pojazdu
pub async fn unassign_vehicle(
    State(state): State<AppState>,
    Path(driver_id): Path<Uuid>,
) -> Result<Json<DriverResponse>, AppError> {
    // Pobierz ID przypisanego pojazdu
    let vehicle_id: Option<Uuid> = sqlx::query_scalar(
        "SELECT assigned_vehicle_id FROM drivers WHERE id = $1"
    )
    .bind(driver_id)
    .fetch_one(&state.db)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    if vehicle_id.is_none() {
        return Err(AppError::ValidationError(
            "Kierowca nie ma przypisanego pojazdu".to_string()
        ));
    }

    // Użyj funkcji SQL do usunięcia przypisania
    let success: bool = sqlx::query_scalar(
        "SELECT unassign_driver_from_vehicle($1)"
    )
    .bind(vehicle_id.unwrap())
    .fetch_one(&state.db)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    if !success {
        return Err(AppError::InternalError(
            "Nie udało się usunąć przypisania".to_string()
        ));
    }

    // Pobierz zaktualizowanego kierowcę
    let driver = sqlx::query_as::<_, Driver>(
        r#"
        SELECT 
            id, user_id, employee_id, license_number, license_categories,
            license_expiry, phone, emergency_contact, status as "status: _",
            assigned_vehicle_id, created_at, updated_at
        FROM drivers
        WHERE id = $1
        "#
    )
    .bind(driver_id)
    .fetch_one(&state.db)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    Ok(Json(DriverResponse::from(driver)))
}

/// GET /drivers/:id/dashboard - dashboard kierowcy (mobile)
pub async fn get_driver_dashboard(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<DriverDashboardInfo>, AppError> {
    // Pobierz podstawowe dane kierowcy
    let row = sqlx::query(
        r#"
        SELECT 
            d.id, CONCAT(u.first_name, ' ', u.last_name) as full_name, d.phone,
            d.assigned_vehicle_id
        FROM drivers d
        JOIN users u ON d.user_id = u.id
        WHERE d.id = $1
        "#
    )
    .bind(id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?
    .ok_or_else(|| AppError::NotFound(format!("Kierowca o ID {} nie istnieje", id)))?;

    let driver_id: Uuid = row.get("id");
    let full_name: String = row.get("full_name");
    let phone: String = row.get("phone");
    let assigned_vehicle_id: Option<Uuid> = row.get("assigned_vehicle_id");

    // Pobierz info o pojeździe
    let assigned_vehicle = if let Some(vid) = assigned_vehicle_id {
        sqlx::query(
            "SELECT id, registration_number, brand, model, capacity FROM vehicles WHERE id = $1"
        )
        .bind(vid)
        .fetch_optional(&state.db)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?
        .map(|v| AssignedVehicleInfo {
            id: v.get("id"),
            registration_number: v.get("registration_number"),
            brand: v.get("brand"),
            model: v.get("model"),
            capacity: v.get("capacity"),
        })
    } else {
        None
    };

    // Pobierz aktualną trasę
    let current_route = if let Some(ref v) = assigned_vehicle {
        sqlx::query(
            r#"
            SELECT r.id, r.name, r.number, r.color,
                (SELECT name FROM stops s 
                 JOIN route_stops rs ON s.id = rs.stop_id 
                 WHERE rs.route_id = r.id ORDER BY rs.stop_order LIMIT 1) as first_stop,
                (SELECT name FROM stops s 
                 JOIN route_stops rs ON s.id = rs.stop_id 
                 WHERE rs.route_id = r.id ORDER BY rs.stop_order DESC LIMIT 1) as last_stop
            FROM routes r
            JOIN vehicles v ON v.current_route_id = r.id
            WHERE v.id = $1
            "#
        )
        .bind(v.id)
        .fetch_optional(&state.db)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?
        .map(|r| DriverRouteInfo {
            id: r.get("id"),
            name: r.get("name"),
            number: r.get("number"),
            color: r.get("color"),
            first_stop: r.get("first_stop"),
            last_stop: r.get("last_stop"),
        })
    } else {
        None
    };

    // TODO: Pobierz dzisiejsze zmiany/grafik kierowcy
    let today_shifts = vec![];

    Ok(Json(DriverDashboardInfo {
        driver_id,
        full_name,
        phone,
        assigned_vehicle,
        current_route,
        today_shifts,
    }))
}

/// GET /drivers/available - dostępni kierowcy (bez przypisanego pojazdu)
pub async fn get_available_drivers(
    State(state): State<AppState>,
) -> Result<Json<DriversListResponse>, AppError> {
    let rows = sqlx::query(
        r#"
        SELECT 
            d.id, d.user_id, d.employee_id, d.license_number,
            d.license_categories, d.license_expiry, d.phone,
            d.emergency_contact, d.status as "status: _",
            d.assigned_vehicle_id, d.created_at, d.updated_at,
            u.first_name, u.last_name, u.email
        FROM drivers d
        JOIN users u ON d.user_id = u.id
        WHERE d.status = 'active'
          AND d.assigned_vehicle_id IS NULL
        ORDER BY u.last_name, u.first_name
        "#
    )
    .fetch_all(&state.db)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    let mut drivers = Vec::new();

    for row in rows {
        let driver = Driver {
            id: row.get("id"),
            user_id: row.get("user_id"),
            employee_id: row.get("employee_id"),
            license_number: row.get("license_number"),
            license_categories: row.get("license_categories"),
            license_expiry: row.get("license_expiry"),
            phone: row.get("phone"),
            emergency_contact: row.get("emergency_contact"),
            status: row.get("status"),
            assigned_vehicle_id: row.get("assigned_vehicle_id"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        };

        let driver_with_user = DriverWithUser {
            driver: DriverResponse::from(driver),
            first_name: row.get("first_name"),
            last_name: row.get("last_name"),
            email: row.get("email"),
            full_name: format!("{} {}", 
                row.get::<String, _>("first_name"), 
                row.get::<String, _>("last_name")
            ),
        };

        drivers.push(driver_with_user);
    }

    let total = drivers.len() as i64;

    Ok(Json(DriversListResponse {
        drivers,
        total,
    }))
}
