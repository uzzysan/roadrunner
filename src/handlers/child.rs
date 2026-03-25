//! Handler dla dzieci (school transport)
//!
//! Endpointy:
//! - GET /children - lista dzieci (rodzic widzi swoje)
//! - POST /children - zarejestruj dziecko
//! - GET /children/:id - szczegóły dziecka
//! - PUT /children/:id - edytuj dane dziecka
//! - DELETE /children/:id - usuń rejestrację
//! - GET /children/:id/qr - pobierz kod QR
//! - GET /children/:id/attendance - historia obecności
//! - GET /children/route/:id - lista dzieci na trasie (dla kierowcy)

use axum::{
    extract::{Path, Query, State},
    Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    models::child::{  // Zakładam że modele są w models/child.rs
        ChildRegistration, ChildResponse, CreateChildRequest, UpdateChildRequest,
        ChildAttendance, AttendanceResponse, ChildWithDetails,
    },
    state::AppState,
    errors::AppError,
};

/// Query parameters dla listy dzieci
#[derive(Debug, Deserialize)]
pub struct ChildrenQuery {
    pub parent_id: Option<Uuid>,
    pub route_id: Option<Uuid>,
    pub status: Option<String>,
}

/// Response dla listy dzieci
#[derive(Debug, Serialize)]
pub struct ChildrenListResponse {
    pub children: Vec<ChildWithDetails>,
    pub total: i64,
}

/// GET /children - lista dzieci
/// Rodzic widzi tylko swoje dzieci, admin/kierowca może filtrować
pub async fn list_children(
    State(state): State<AppState>,
    Query(query): Query<ChildrenQuery>,
) -> Result<Json<ChildrenListResponse>, AppError> {
    let mut sql = String::from(
        r#"
        SELECT 
            cr.id, cr.parent_user_id, cr.child_first_name, cr.child_last_name,
            cr.child_birth_date, cr.school_name, cr.school_address,
            cr.assigned_route_id, cr.pickup_stop_id, cr.dropoff_stop_id,
            cr.qr_code, cr.qr_code_data, cr.photo_url, cr.status as "status: _",
            cr.notes, cr.created_at, cr.updated_at,
            u.first_name as parent_first_name, u.last_name as parent_last_name,
            u.email as parent_email, u.phone as parent_phone,
            r.name as route_name, r.number as route_number, r.color as route_color,
            pickup.name as pickup_stop_name,
            dropoff.name as dropoff_stop_name
        FROM child_registrations cr
        JOIN users u ON cr.parent_user_id = u.id
        LEFT JOIN routes r ON cr.assigned_route_id = r.id
        LEFT JOIN stops pickup ON cr.pickup_stop_id = pickup.id
        LEFT JOIN stops dropoff ON cr.dropoff_stop_id = dropoff.id
        WHERE 1=1
        "#
    );

    if let Some(parent_id) = query.parent_id {
        sql.push_str(&format!(" AND cr.parent_user_id = '{}'", parent_id));
    }

    if let Some(route_id) = query.route_id {
        sql.push_str(&format!(" AND cr.assigned_route_id = '{}'", route_id));
    }

    if let Some(ref status) = query.status {
        sql.push_str(&format!(" AND cr.status = '{}'", status));
    }

    sql.push_str(" ORDER BY cr.child_last_name, cr.child_first_name");

    let rows = sqlx::query(&sql)
        .fetch_all(&state.db)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    let children: Vec<ChildWithDetails> = rows.into_iter()
        .map(|row| row_to_child_with_details(row))
        .collect();

    let total = children.len() as i64;

    Ok(Json(ChildrenListResponse {
        children,
        total,
    }))
}

/// POST /children - zarejestruj nowe dziecko
pub async fn create_child(
    State(state): State<AppState>,
    Json(request): Json<CreateChildRequest>,
) -> Result<Json<ChildWithDetails>, AppError> {
    // Walidacja
    if request.child_first_name.trim().is_empty() || request.child_last_name.trim().is_empty() {
        return Err(AppError::ValidationError(
            "Imię i nazwisko dziecka są wymagane".to_string()
        ));
    }

    if request.school_name.trim().is_empty() {
        return Err(AppError::ValidationError(
            "Nazwa szkoły jest wymagana".to_string()
        ));
    }

    // Sprawdź czy rodzic istnieje
    let parent_exists: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM users WHERE id = $1)"
    )
    .bind(request.parent_user_id)
    .fetch_one(&state.db)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    if !parent_exists {
        return Err(AppError::NotFound("Rodzic nie istnieje".to_string()));
    }

    // Użyj funkcji SQL do rejestracji dziecka
    let child_id: Uuid = sqlx::query_scalar(
        "SELECT register_child($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)"
    )
    .bind(request.parent_user_id)
    .bind(&request.child_first_name)
    .bind(&request.child_last_name)
    .bind(request.child_birth_date)
    .bind(&request.school_name)
    .bind(&request.school_address)
    .bind(request.assigned_route_id)
    .bind(request.pickup_stop_id)
    .bind(request.dropoff_stop_id)
    .bind(&request.notes)
    .fetch_one(&state.db)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    // Pobierz utworzone dziecko
    let child = get_child_by_id(&state, child_id).await?;

    Ok(Json(child))
}

/// GET /children/:id - szczegóły dziecka
pub async fn get_child(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<ChildWithDetails>, AppError> {
    let child = get_child_by_id(&state, id).await?;
    Ok(Json(child))
}

/// PUT /children/:id - edytuj dane dziecka
pub async fn update_child(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(request): Json<UpdateChildRequest>,
) -> Result<Json<ChildWithDetails>, AppError> {
    // Sprawdź czy dziecko istnieje
    let exists: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM child_registrations WHERE id = $1)"
    )
    .bind(id)
    .fetch_one(&state.db)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    if !exists {
        return Err(AppError::NotFound(format!("Dziecko o ID {} nie istnieje", id)));
    }

    // Aktualizuj dane
    sqlx::query(
        r#"
        UPDATE child_registrations
        SET 
            child_first_name = COALESCE($1, child_first_name),
            child_last_name = COALESCE($2, child_last_name),
            child_birth_date = COALESCE($3, child_birth_date),
            school_name = COALESCE($4, school_name),
            school_address = COALESCE($5, school_address),
            assigned_route_id = COALESCE($6, assigned_route_id),
            pickup_stop_id = COALESCE($7, pickup_stop_id),
            dropoff_stop_id = COALESCE($8, dropoff_stop_id),
            photo_url = COALESCE($9, photo_url),
            status = COALESCE($10, status),
            notes = COALESCE($11, notes),
            updated_at = NOW()
        WHERE id = $12
        "#
    )
    .bind(&request.child_first_name)
    .bind(&request.child_last_name)
    .bind(request.child_birth_date)
    .bind(&request.school_name)
    .bind(&request.school_address)
    .bind(request.assigned_route_id)
    .bind(request.pickup_stop_id)
    .bind(request.dropoff_stop_id)
    .bind(&request.photo_url)
    .bind(&request.status)
    .bind(&request.notes)
    .bind(id)
    .execute(&state.db)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    // Pobierz zaktualizowane dziecko
    let child = get_child_by_id(&state, id).await?;

    Ok(Json(child))
}

/// DELETE /children/:id - usuń rejestrację dziecka
pub async fn delete_child(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    let result = sqlx::query("DELETE FROM child_registrations WHERE id = $1")
        .bind(id)
        .execute(&state.db)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound(format!("Dziecko o ID {} nie istnieje", id)));
    }

    Ok(Json(serde_json::json!({
        "message": "Rejestracja dziecka została usunięta",
        "child_id": id
    })))
}

/// GET /children/:id/qr - pobierz kod QR dziecka
#[derive(Debug, Serialize)]
pub struct QRCodeResponse {
    pub child_id: Uuid,
    pub child_name: String,
    pub qr_code: String,
    pub qr_code_data: String,
    pub qr_image_url: Option<String>, // URL do wygenerowanego obrazu QR
}

pub async fn get_child_qr(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<QRCodeResponse>, AppError> {
    let row = sqlx::query(
        r#"
        SELECT 
            cr.id, cr.qr_code, cr.qr_code_data,
            cr.child_first_name || ' ' || cr.child_last_name as child_name
        FROM child_registrations cr
        WHERE cr.id = $1
        "#
    )
    .bind(id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?
    .ok_or_else(|| AppError::NotFound(format!("Dziecko o ID {} nie istnieje", id)))?;

    Ok(Json(QRCodeResponse {
        child_id: row.get("id"),
        child_name: row.get("child_name"),
        qr_code: row.get("qr_code"),
        qr_code_data: row.get("qr_code_data"),
        qr_image_url: None, // TODO: Generowanie obrazu QR
    }))
}

/// GET /children/:id/attendance - historia obecności dziecka
#[derive(Debug, Serialize)]
pub struct AttendanceListResponse {
    pub attendance: Vec<AttendanceResponse>,
    pub total: i64,
}

pub async fn get_child_attendance(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Query(query): Query<AttendanceQuery>,
) -> Result<Json<AttendanceListResponse>, AppError> {
    let limit = query.limit.unwrap_or(30);
    let offset = query.offset.unwrap_or(0);

    let rows = sqlx::query(
        r#"
        SELECT 
            ca.id, ca.child_id, ca.route_id, ca.vehicle_id, ca.driver_id,
            ca.pickup_stop_id, ca.pickup_time, ca.dropoff_stop_id, ca.dropoff_time,
            ca.status as "status: _", ca.confirmed_by as "confirmed_by: _",
            ca.parent_notified_pickup, ca.parent_notified_dropoff, ca.date,
            r.name as route_name, r.number as route_number,
            v.registration_number as vehicle_registration,
            CONCAT(u.first_name, ' ', u.last_name) as driver_name,
            pickup.name as pickup_stop_name,
            dropoff.name as dropoff_stop_name
        FROM child_attendance ca
        JOIN routes r ON ca.route_id = r.id
        JOIN vehicles v ON ca.vehicle_id = v.id
        JOIN drivers d ON ca.driver_id = d.id
        JOIN users u ON d.user_id = u.id
        LEFT JOIN stops pickup ON ca.pickup_stop_id = pickup.id
        LEFT JOIN stops dropoff ON ca.dropoff_stop_id = dropoff.id
        WHERE ca.child_id = $1
        ORDER BY ca.date DESC, ca.pickup_time DESC
        LIMIT $2 OFFSET $3
        "#
    )
    .bind(id)
    .bind(limit)
    .bind(offset)
    .fetch_all(&state.db)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    let attendance: Vec<AttendanceResponse> = rows.into_iter()
        .map(|row| AttendanceResponse {
            id: row.get("id"),
            child_id: row.get("child_id"),
            route_id: row.get("route_id"),
            route_name: row.get("route_name"),
            route_number: row.get("route_number"),
            vehicle_id: row.get("vehicle_id"),
            vehicle_registration: row.get("vehicle_registration"),
            driver_id: row.get("driver_id"),
            driver_name: row.get("driver_name"),
            pickup_stop_id: row.get("pickup_stop_id"),
            pickup_stop_name: row.get("pickup_stop_name"),
            pickup_time: row.get("pickup_time"),
            dropoff_stop_id: row.get("dropoff_stop_id"),
            dropoff_stop_name: row.get("dropoff_stop_name"),
            dropoff_time: row.get("dropoff_time"),
            status: row.get("status"),
            confirmed_by: row.get("confirmed_by"),
            parent_notified_pickup: row.get("parent_notified_pickup"),
            parent_notified_dropoff: row.get("parent_notified_dropoff"),
            date: row.get("date"),
        })
        .collect();

    let total = attendance.len() as i64;

    Ok(Json(AttendanceListResponse {
        attendance,
        total,
    }))
}

#[derive(Debug, Deserialize)]
pub struct AttendanceQuery {
    pub limit: Option<i32>,
    pub offset: Option<i32>,
}

/// GET /children/route/:id - lista dzieci na trasie (dla kierowcy)
pub async fn get_children_by_route(
    State(state): State<AppState>,
    Path(route_id): Path<Uuid>,
) -> Result<Json<ChildrenListResponse>, AppError> {
    let rows = sqlx::query(
        r#"
        SELECT 
            cr.id, cr.parent_user_id, cr.child_first_name, cr.child_last_name,
            cr.child_birth_date, cr.school_name, cr.school_address,
            cr.assigned_route_id, cr.pickup_stop_id, cr.dropoff_stop_id,
            cr.qr_code, cr.qr_code_data, cr.photo_url, cr.status as "status: _",
            cr.notes, cr.created_at, cr.updated_at,
            u.first_name as parent_first_name, u.last_name as parent_last_name,
            u.email as parent_email, u.phone as parent_phone,
            r.name as route_name, r.number as route_number, r.color as route_color,
            pickup.name as pickup_stop_name,
            dropoff.name as dropoff_stop_name
        FROM child_registrations cr
        JOIN users u ON cr.parent_user_id = u.id
        LEFT JOIN routes r ON cr.assigned_route_id = r.id
        LEFT JOIN stops pickup ON cr.pickup_stop_id = pickup.id
        LEFT JOIN stops dropoff ON cr.dropoff_stop_id = dropoff.id
        WHERE cr.assigned_route_id = $1
          AND cr.status = 'active'
        ORDER BY pickup.name, cr.child_last_name, cr.child_first_name
        "#
    )
    .bind(route_id)
    .fetch_all(&state.db)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    let children: Vec<ChildWithDetails> = rows.into_iter()
        .map(|row| row_to_child_with_details(row))
        .collect();

    let total = children.len() as i64;

    Ok(Json(ChildrenListResponse {
        children,
        total,
    }))
}

// Helper functions

async fn get_child_by_id(state: &AppState, id: Uuid) -> Result<ChildWithDetails, AppError> {
    let row = sqlx::query(
        r#"
        SELECT 
            cr.id, cr.parent_user_id, cr.child_first_name, cr.child_last_name,
            cr.child_birth_date, cr.school_name, cr.school_address,
            cr.assigned_route_id, cr.pickup_stop_id, cr.dropoff_stop_id,
            cr.qr_code, cr.qr_code_data, cr.photo_url, cr.status as "status: _",
            cr.notes, cr.created_at, cr.updated_at,
            u.first_name as parent_first_name, u.last_name as parent_last_name,
            u.email as parent_email, u.phone as parent_phone,
            r.name as route_name, r.number as route_number, r.color as route_color,
            pickup.name as pickup_stop_name,
            dropoff.name as dropoff_stop_name
        FROM child_registrations cr
        JOIN users u ON cr.parent_user_id = u.id
        LEFT JOIN routes r ON cr.assigned_route_id = r.id
        LEFT JOIN stops pickup ON cr.pickup_stop_id = pickup.id
        LEFT JOIN stops dropoff ON cr.dropoff_stop_id = dropoff.id
        WHERE cr.id = $1
        "#
    )
    .bind(id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?
    .ok_or_else(|| AppError::NotFound(format!("Dziecko o ID {} nie istnieje", id)))?;

    Ok(row_to_child_with_details(row))
}

fn row_to_child_with_details(row: sqlx::postgres::PgRow) -> ChildWithDetails {
    use sqlx::Row;

    ChildWithDetails {
        id: row.get("id"),
        parent_user_id: row.get("parent_user_id"),
        child_first_name: row.get("child_first_name"),
        child_last_name: row.get("child_last_name"),
        child_full_name: format!("{} {}",
            row.get::<String, _>("child_first_name"),
            row.get::<String, _>("child_last_name")
        ),
        child_birth_date: row.get("child_birth_date"),
        school_name: row.get("school_name"),
        school_address: row.get("school_address"),
        assigned_route_id: row.get("assigned_route_id"),
        route_name: row.get("route_name"),
        route_number: row.get("route_number"),
        route_color: row.get("route_color"),
        pickup_stop_id: row.get("pickup_stop_id"),
        pickup_stop_name: row.get("pickup_stop_name"),
        dropoff_stop_id: row.get("dropoff_stop_id"),
        dropoff_stop_name: row.get("dropoff_stop_name"),
        qr_code: row.get("qr_code"),
        qr_code_data: row.get("qr_code_data"),
        photo_url: row.get("photo_url"),
        status: row.get("status"),
        notes: row.get("notes"),
        parent_first_name: row.get("parent_first_name"),
        parent_last_name: row.get("parent_last_name"),
        parent_email: row.get("parent_email"),
        parent_phone: row.get("parent_phone"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}
