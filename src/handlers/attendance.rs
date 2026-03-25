//! Handler dla obecności dzieci (attendance)
//!
//! Endpointy:
//! - GET /attendance/today - dzisiejsza lista obecności (dla kierowcy)
//! - POST /attendance/scan - skanuj kod QR (wsiadanie/wysiadanie)
//! - POST /attendance/manual - manualne potwierdzenie
//! - GET /attendance/:child_id/status - status dziecka
//! - POST /attendance/:child_id/absent - oznacz jako nieobecne

use axum::{
    extract::{Path, Query, State},
    Json,
};
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    models::child::{ChildAttendance, AttendanceStatus, ConfirmationMethod, AttendanceResponse},
    state::AppState,
    errors::AppError,
};

/// Request do skanowania QR
#[derive(Debug, Deserialize)]
pub struct ScanQRRequest {
    pub qr_code: String,
    pub stop_id: Uuid,
    pub action: ScanAction, // pickup lub dropoff
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScanAction {
    Pickup,   // Wsiadanie
    Dropoff,  // Wysiadanie
}

/// Response po skanowaniu
#[derive(Debug, Serialize)]
pub struct ScanResponse {
    pub success: bool,
    pub child_id: Uuid,
    pub child_name: String,
    pub action: String,
    pub stop_name: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub parent_notified: bool,
}

/// GET /attendance/today - dzisiejsza lista obecności dla kierowcy
#[derive(Debug, Serialize)]
pub struct TodayAttendanceResponse {
    pub date: NaiveDate,
    pub route_id: Uuid,
    pub route_name: String,
    pub total_children: i64,
    pub picked_up: i64,
    pub dropped_off: i64,
    pub absent: i64,
    pub children: Vec<TodayAttendanceItem>,
}

#[derive(Debug, Serialize)]
pub struct TodayAttendanceItem {
    pub attendance_id: Uuid,
    pub child_id: Uuid,
    pub child_name: String,
    pub photo_url: Option<String>,
    pub pickup_stop_name: String,
    pub dropoff_stop_name: String,
    pub status: AttendanceStatus,
    pub pickup_time: Option<chrono::DateTime<chrono::Utc>>,
    pub dropoff_time: Option<chrono::DateTime<chrono::Utc>>,
    pub parent_notified_pickup: bool,
    pub parent_notified_dropoff: bool,
}

pub async fn get_today_attendance(
    State(state): State<AppState>,
    Query(query): Query<TodayAttendanceQuery>,
) -> Result<Json<TodayAttendanceResponse>, AppError> {
    let route_id = query.route_id.ok_or_else(|| {
        AppError::ValidationError("Parametr route_id jest wymagany".to_string())
    })?;

    let date = query.date.unwrap_or_else(|| chrono::Local::now().date_naive());

    // Pobierz dane trasy
    let route = sqlx::query("SELECT id, name, number FROM routes WHERE id = $1")
        .bind(route_id)
        .fetch_one(&state.db)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    // Pobierz listę dzieci na dzisiaj
    let rows = sqlx::query(
        r#"
        SELECT 
            ca.id as attendance_id, ca.child_id, ca.status as "status: _",
            ca.pickup_time, ca.dropoff_time, ca.parent_notified_pickup,
            ca.parent_notified_dropoff,
            cr.child_first_name || ' ' || cr.child_last_name as child_name,
            cr.photo_url,
            pickup.name as pickup_stop_name,
            dropoff.name as dropoff_stop_name
        FROM child_attendance ca
        JOIN child_registrations cr ON ca.child_id = cr.id
        LEFT JOIN stops pickup ON ca.pickup_stop_id = pickup.id
        LEFT JOIN stops dropoff ON ca.dropoff_stop_id = dropoff.id
        WHERE ca.route_id = $1
          AND ca.date = $2
        ORDER BY pickup.name, child_name
        "#
    )
    .bind(route_id)
    .bind(date)
    .fetch_all(&state.db)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    let mut children = Vec::new();
    let mut picked_up = 0i64;
    let mut dropped_off = 0i64;
    let mut absent = 0i64;

    for row in &rows {
        let status: AttendanceStatus = row.get("status");
        match status {
            AttendanceStatus::PickedUp => picked_up += 1,
            AttendanceStatus::DroppedOff => dropped_off += 1,
            AttendanceStatus::Absent => absent += 1,
            _ => {}
        }

        children.push(TodayAttendanceItem {
            attendance_id: row.get("attendance_id"),
            child_id: row.get("child_id"),
            child_name: row.get("child_name"),
            photo_url: row.get("photo_url"),
            pickup_stop_name: row.get("pickup_stop_name"),
            dropoff_stop_name: row.get("dropoff_stop_name"),
            status,
            pickup_time: row.get("pickup_time"),
            dropoff_time: row.get("dropoff_time"),
            parent_notified_pickup: row.get("parent_notified_pickup"),
            parent_notified_dropoff: row.get("parent_notified_dropoff"),
        });
    }

    Ok(Json(TodayAttendanceResponse {
        date,
        route_id: route.get("id"),
        route_name: format!("{} {}", route.get::<String, _>("number"), route.get::<String, _>("name")),
        total_children: children.len() as i64,
        picked_up,
        dropped_off,
        absent,
        children,
    }))
}

#[derive(Debug, Deserialize)]
pub struct TodayAttendanceQuery {
    pub route_id: Option<Uuid>,
    pub date: Option<NaiveDate>,
}

/// POST /attendance/scan - skanuj kod QR
pub async fn scan_qr(
    State(state): State<AppState>,
    Json(request): Json<ScanQRRequest>,
) -> Result<Json<ScanResponse>, AppError> {
    // Znajdź dziecko po kodzie QR
    let child = sqlx::query(
        r#"
        SELECT 
            cr.id, cr.child_first_name || ' ' || cr.child_last_name as child_name,
            cr.assigned_route_id
        FROM child_registrations cr
        WHERE cr.qr_code = $1 AND cr.status = 'active'
        "#
    )
    .bind(&request.qr_code)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?
    .ok_or_else(|| AppError::NotFound("Nieprawidłowy kod QR".to_string()))?;

    let child_id: Uuid = child.get("id");
    let child_name: String = child.get("child_name");

    // Pobierz lub utwórz rekord obecności na dzisiaj
    let today = chrono::Local::now().date_naive();

    let attendance = sqlx::query(
        r#"
        SELECT id, status as "status: _"
        FROM child_attendance
        WHERE child_id = $1 AND date = $2
        "#
    )
    .bind(child_id)
    .bind(today)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    let attendance_id: Uuid;
    let current_status: Option<AttendanceStatus>;

    if let Some(row) = attendance {
        attendance_id = row.get("id");
        current_status = row.get("status");
    } else {
        // Utwórz nowy rekord obecności
        let route_id: Uuid = child.get("assigned_route_id");

        // Pobierz pojazd i kierowcę dla trasy
        let vehicle = sqlx::query(
            "SELECT id, current_driver_id FROM vehicles WHERE current_route_id = $1 LIMIT 1"
        )
        .bind(route_id)
        .fetch_one(&state.db)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        let new_attendance = sqlx::query(
            r#"
            INSERT INTO child_attendance (
                child_id, route_id, vehicle_id, driver_id, date, status
            ) VALUES ($1, $2, $3, $4, $5, 'scheduled')
            RETURNING id
            "#
        )
        .bind(child_id)
        .bind(route_id)
        .bind(vehicle.get::<Uuid, _>("id"))
        .bind(vehicle.get::<Option<Uuid>, _>("current_driver_id"))
        .bind(today)
        .fetch_one(&state.db)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        attendance_id = new_attendance.get("id");
        current_status = Some(AttendanceStatus::Scheduled);
    }

    // Pobierz nazwę przystanku
    let stop_name: String = sqlx::query_scalar("SELECT name FROM stops WHERE id = $1")
        .bind(request.stop_id)
        .fetch_one(&state.db)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    // Wykonaj odpowiednią akcję
    let (action_str, parent_notified) = match request.action {
        ScanAction::Pickup => {
            // Sprawdź czy dziecko nie zostało już odebrane
            if let Some(status) = current_status {
                if status == AttendanceStatus::PickedUp || status == AttendanceStatus::DroppedOff {
                    return Err(AppError::ValidationError(
                        "Dziecko zostało już odebrane".to_string()
                    ));
                }
            }

            // Użyj funkcji SQL do potwierdzenia wsiadania
            let notified: bool = sqlx::query_scalar(
                "SELECT confirm_child_pickup($1, $2, 'qr_code')"
            )
            .bind(attendance_id)
            .bind(request.stop_id)
            .fetch_one(&state.db)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

            ("pickup", notified)
        }
        ScanAction::Dropoff => {
            // Sprawdź czy dziecko zostało odebrane
            if let Some(status) = current_status {
                if status != AttendanceStatus::PickedUp {
                    return Err(AppError::ValidationError(
                        "Dziecko nie zostało jeszcze odebrane".to_string()
                    ));
                }
                if status == AttendanceStatus::DroppedOff {
                    return Err(AppError::ValidationError(
                        "Dziecko zostało już dostarczone".to_string()
                    ));
                }
            }

            // Użyj funkcji SQL do potwierdzenia wysiadania
            let notified: bool = sqlx::query_scalar(
                "SELECT confirm_child_dropoff($1, $2, 'qr_code')"
            )
            .bind(attendance_id)
            .bind(request.stop_id)
            .fetch_one(&state.db)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

            ("dropoff", notified)
        }
    };

    Ok(Json(ScanResponse {
        success: true,
        child_id,
        child_name,
        action: action_str.to_string(),
        stop_name,
        timestamp: chrono::Utc::now(),
        parent_notified,
    }))
}

/// POST /attendance/manual - manualne potwierdzenie (bez QR)
#[derive(Debug, Deserialize)]
pub struct ManualAttendanceRequest {
    pub child_id: Uuid,
    pub stop_id: Uuid,
    pub action: ScanAction,
}

pub async fn manual_confirm(
    State(state): State<AppState>,
    Json(request): Json<ManualAttendanceRequest>,
) -> Result<Json<ScanResponse>, AppError> {
    // Pobierz dane dziecka
    let child = sqlx::query(
        "SELECT child_first_name || ' ' || child_last_name as child_name FROM child_registrations WHERE id = $1"
    )
    .bind(request.child_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?
    .ok_or_else(|| AppError::NotFound("Dziecko nie istnieje".to_string()))?;

    let child_name: String = child.get("child_name");

    // Pobierz lub utwórz rekord obecności
    let today = chrono::Local::now().date_naive();

    let attendance = sqlx::query(
        "SELECT id FROM child_attendance WHERE child_id = $1 AND date = $2"
    )
    .bind(request.child_id)
    .bind(today)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    let attendance_id: Uuid = if let Some(row) = attendance {
        row.get("id")
    } else {
        return Err(AppError::NotFound(
            "Nie znaleziono dzisiejszego rekordu obecności".to_string()
        ));
    };

    // Pobierz nazwę przystanku
    let stop_name: String = sqlx::query_scalar("SELECT name FROM stops WHERE id = $1")
        .bind(request.stop_id)
        .fetch_one(&state.db)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    // Wykonaj akcję
    let (action_str, parent_notified) = match request.action {
        ScanAction::Pickup => {
            let notified: bool = sqlx::query_scalar(
                "SELECT confirm_child_pickup($1, $2, 'manual')"
            )
            .bind(attendance_id)
            .bind(request.stop_id)
            .fetch_one(&state.db)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

            ("pickup", notified)
        }
        ScanAction::Dropoff => {
            let notified: bool = sqlx::query_scalar(
                "SELECT confirm_child_dropoff($1, $2, 'manual')"
            )
            .bind(attendance_id)
            .bind(request.stop_id)
            .fetch_one(&state.db)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

            ("dropoff", notified)
        }
    };

    Ok(Json(ScanResponse {
        success: true,
        child_id: request.child_id,
        child_name,
        action: action_str.to_string(),
        stop_name,
        timestamp: chrono::Utc::now(),
        parent_notified,
    }))
}

/// GET /attendance/:child_id/status - status dziecka
#[derive(Debug, Serialize)]
pub struct ChildStatusResponse {
    pub child_id: Uuid,
    pub child_name: String,
    pub date: NaiveDate,
    pub status: AttendanceStatus,
    pub pickup_time: Option<chrono::DateTime<chrono::Utc>>,
    pub dropoff_time: Option<chrono::DateTime<chrono::Utc>>,
    pub pickup_stop_name: Option<String>,
    pub dropoff_stop_name: Option<String>,
    pub parent_notified_pickup: bool,
    pub parent_notified_dropoff: bool,
}

pub async fn get_child_status(
    State(state): State<AppState>,
    Path(child_id): Path<Uuid>,
    Query(query): Query<StatusQuery>,
) -> Result<Json<ChildStatusResponse>, AppError> {
    let date = query.date.unwrap_or_else(|| chrono::Local::now().date_naive());

    let row = sqlx::query(
        r#"
        SELECT 
            ca.child_id,
            cr.child_first_name || ' ' || cr.child_last_name as child_name,
            ca.date, ca.status as "status: _",
            ca.pickup_time, ca.dropoff_time,
            ca.parent_notified_pickup, ca.parent_notified_dropoff,
            pickup.name as pickup_stop_name,
            dropoff.name as dropoff_stop_name
        FROM child_attendance ca
        JOIN child_registrations cr ON ca.child_id = cr.id
        LEFT JOIN stops pickup ON ca.pickup_stop_id = pickup.id
        LEFT JOIN stops dropoff ON ca.dropoff_stop_id = dropoff.id
        WHERE ca.child_id = $1 AND ca.date = $2
        "#
    )
    .bind(child_id)
    .bind(date)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?
    .ok_or_else(|| AppError::NotFound("Nie znaleziono rekordu obecności".to_string()))?;

    Ok(Json(ChildStatusResponse {
        child_id: row.get("child_id"),
        child_name: row.get("child_name"),
        date: row.get("date"),
        status: row.get("status"),
        pickup_time: row.get("pickup_time"),
        dropoff_time: row.get("dropoff_time"),
        pickup_stop_name: row.get("pickup_stop_name"),
        dropoff_stop_name: row.get("dropoff_stop_name"),
        parent_notified_pickup: row.get("parent_notified_pickup"),
        parent_notified_dropoff: row.get("parent_notified_dropoff"),
    }))
}

#[derive(Debug, Deserialize)]
pub struct StatusQuery {
    pub date: Option<NaiveDate>,
}

/// POST /attendance/:child_id/absent - oznacz dziecko jako nieobecne
pub async fn mark_absent(
    State(state): State<AppState>,
    Path(child_id): Path<Uuid>,
    Query(query): Query<AbsentQuery>,
) -> Result<Json<serde_json::Value>, AppError> {
    let date = query.date.unwrap_or_else(|| chrono::Local::now().date_naive());

    let success: bool = sqlx::query_scalar(
        "SELECT mark_child_absent($1, $2)"
    )
    .bind(child_id)
    .bind(date)
    .fetch_one(&state.db)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    if !success {
        return Err(AppError::NotFound("Nie znaleziono rekordu obecności".to_string()));
    }

    Ok(Json(serde_json::json!({
        "message": "Dziecko zostało oznaczone jako nieobecne",
        "child_id": child_id,
        "date": date
    })))
}

#[derive(Debug, Deserialize)]
pub struct AbsentQuery {
    pub date: Option<NaiveDate>,
}
