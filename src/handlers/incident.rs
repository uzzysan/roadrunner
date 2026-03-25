//! Handler dla incydentów (incidents) - system awarii
//!
//! Endpointy:
//! - GET /incidents - lista incydentów
//! - POST /incidents - zgłoś incydent (kierowca)
//! - GET /incidents/:id - szczegóły incydentu
//! - PUT /incidents/:id - aktualizuj incydent
//! - POST /incidents/:id/resolve - rozwiąż incydent
//! - POST /incidents/:id/assign-replacement - przypisz pojazd zastępczy
//! - GET /incidents/active - aktywne incydenty
//! - GET /incidents/stats - statystyki

use axum::{
    extract::{Path, Query, State},
    Json,
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    models::incident::{
        Incident, IncidentResponse, IncidentNotification, IncidentStats,
        SeverityCount, TypeCount, CreateIncidentRequest, UpdateIncidentRequest,
        ResolveIncidentRequest, AssignReplacementVehicleRequest,
        IncidentType, Severity, IncidentStatus, generate_passenger_notification,
        calculate_duration_minutes,
    },
    state::AppState,
    errors::AppError,
};

/// Query parameters dla filtrowania incydentów
#[derive(Debug, Deserialize)]
pub struct IncidentsQuery {
    pub status: Option<IncidentStatus>,
    pub incident_type: Option<IncidentType>,
    pub severity: Option<Severity>,
    pub vehicle_id: Option<Uuid>,
    pub route_id: Option<Uuid>,
}

/// Response dla listy incydentów
#[derive(Debug, Serialize)]
pub struct IncidentsListResponse {
    pub incidents: Vec<IncidentResponse>,
    pub total: i64,
}

/// GET /incidents - lista wszystkich incydentów
pub async fn list_incidents(
    State(state): State<AppState>,
    Query(query): Query<IncidentsQuery>,
) -> Result<Json<IncidentsListResponse>, AppError> {
    let mut sql = String::from(
        r#"
        SELECT 
            i.id, i.vehicle_id, i.driver_id, i.incident_type as "incident_type: _",
            i.severity as "severity: _", i.title, i.description,
            ST_AsText(i.location) as location, i.reported_at, i.resolved_at,
            i.resolved_by, i.resolution_notes, i.status as "status: _",
            i.replacement_vehicle_id, i.estimated_resolution, i.created_at, i.updated_at,
            v.registration_number, v.brand as vehicle_brand, v.model as vehicle_model,
            v.current_route_id, r.name as route_name, r.number as route_number, r.color as route_color,
            CONCAT(u.first_name, ' ', u.last_name) as driver_name,
            ru.first_name as resolved_by_first_name, ru.last_name as resolved_by_last_name
        FROM incidents i
        JOIN vehicles v ON i.vehicle_id = v.id
        JOIN drivers d ON i.driver_id = d.id
        JOIN users u ON d.user_id = u.id
        LEFT JOIN routes r ON v.current_route_id = r.id
        LEFT JOIN users ru ON i.resolved_by = ru.id
        WHERE 1=1
        "#
    );

    if let Some(status) = query.status {
        sql.push_str(&format!(" AND i.status = '{:?}'", status).to_lowercase());
    }

    if let Some(ref itype) = query.incident_type {
        sql.push_str(&format!(" AND i.incident_type = '{:?}'", itype).to_lowercase());
    }

    if let Some(severity) = query.severity {
        sql.push_str(&format!(" AND i.severity = '{:?}'", severity).to_lowercase());
    }

    if let Some(vehicle_id) = query.vehicle_id {
        sql.push_str(&format!(" AND i.vehicle_id = '{}'", vehicle_id));
    }

    if let Some(route_id) = query.route_id {
        sql.push_str(&format!(" AND v.current_route_id = '{}'", route_id));
    }

    sql.push_str(" ORDER BY i.reported_at DESC");

    let rows = sqlx::query(&sql)
        .fetch_all(&state.db)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    let incidents: Vec<IncidentResponse> = rows.into_iter()
        .map(|row| row_to_incident_response(row))
        .collect();

    let total = incidents.len() as i64;

    Ok(Json(IncidentsListResponse {
        incidents,
        total,
    }))
}

/// POST /incidents - zgłoś nowy incydent (kierowca)
pub async fn create_incident(
    State(state): State<AppState>,
    Json(request): Json<CreateIncidentRequest>,
) -> Result<Json<IncidentResponse>, AppError> {
    // Walidacja
    if request.title.trim().is_empty() {
        return Err(AppError::ValidationError(
            "Tytuł incydentu jest wymagany".to_string()
        ));
    }

    // Sprawdź czy pojazd istnieje
    let vehicle = sqlx::query(
        "SELECT id, registration_number, brand, model, current_route_id, status FROM vehicles WHERE id = $1"
    )
    .bind(request.vehicle_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?
    .ok_or_else(|| AppError::NotFound("Pojazd nie istnieje".to_string()))?;

    let vehicle_status: String = vehicle.get("status");
    if vehicle_status != "active" {
        return Err(AppError::ValidationError(
            "Nie można zgłosić incydentu dla nieaktywnego pojazdu".to_string()
        ));
    }

    // Sprawdź czy kierowca istnieje
    let driver_exists: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM drivers WHERE id = $1 AND status = 'active')"
    )
    .bind(request.driver_id)
    .fetch_one(&state.db)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    if !driver_exists {
        return Err(AppError::NotFound("Kierowca nie istnieje lub jest nieaktywny".to_string()));
    }

    // Użyj funkcji SQL do utworzenia incydentu
    let incident_id: Uuid = sqlx::query_scalar(
        "SELECT create_incident($1, $2, $3, $4, $5, $6, $7, $8, $9)"
    )
    .bind(request.vehicle_id)
    .bind(request.driver_id)
    .bind(request.incident_type)
    .bind(request.severity)
    .bind(&request.title)
    .bind(&request.description)
    .bind(request.latitude)
    .bind(request.longitude)
    .bind(request.estimated_resolution_minutes)
    .fetch_one(&state.db)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    // Pobierz utworzony incydent
    let incident = get_incident_by_id(&state, incident_id).await?;

    Ok(Json(incident))
}

/// GET /incidents/:id - szczegóły incydentu
pub async fn get_incident(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<IncidentResponse>, AppError> {
    let incident = get_incident_by_id(&state, id).await?;
    Ok(Json(incident))
}

/// PUT /incidents/:id - aktualizuj incydent
pub async fn update_incident(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(request): Json<UpdateIncidentRequest>,
) -> Result<Json<IncidentResponse>, AppError> {
    let incident = sqlx::query_as::<_, Incident>(
        r#"
        UPDATE incidents
        SET 
            title = COALESCE($1, title),
            description = COALESCE($2, description),
            severity = COALESCE($3, severity),
            status = COALESCE($4, status),
            estimated_resolution = COALESCE($5, estimated_resolution),
            updated_at = NOW()
        WHERE id = $6
        RETURNING 
            id, vehicle_id, driver_id, incident_type as "incident_type: _",
            severity as "severity: _", title, description,
            ST_AsText(location) as location, reported_at, resolved_at,
            resolved_by, resolution_notes, status as "status: _",
            replacement_vehicle_id, estimated_resolution, created_at, updated_at
        "#
    )
    .bind(&request.title)
    .bind(&request.description)
    .bind(request.severity)
    .bind(request.status)
    .bind(request.estimated_resolution)
    .bind(id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?
    .ok_or_else(|| AppError::NotFound(format!("Incydent o ID {} nie istnieje", id)))?;

    // Pobierz pełne dane o incydencie
    let incident_response = get_incident_by_id(&state, incident.id).await?;

    Ok(Json(incident_response))
}

/// POST /incidents/:id/resolve - rozwiąż incydent
pub async fn resolve_incident(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(request): Json<ResolveIncidentRequest>,
) -> Result<Json<IncidentResponse>, AppError> {
    // TODO: Pobrać ID użytkownika z tokena JWT
    let resolved_by = Uuid::nil(); // Tymczasowo

    // Użyj funkcji SQL
    let success: bool = sqlx::query_scalar(
        "SELECT resolve_incident($1, $2, $3)"
    )
    .bind(id)
    .bind(resolved_by)
    .bind(&request.resolution_notes)
    .fetch_one(&state.db)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    if !success {
        return Err(AppError::NotFound(format!("Incydent o ID {} nie istnieje", id)));
    }

    // Pobierz zaktualizowany incydent
    let incident = get_incident_by_id(&state, id).await?;

    Ok(Json(incident))
}

/// POST /incidents/:id/assign-replacement - przypisz pojazd zastępczy
pub async fn assign_replacement_vehicle(
    State(state): State<AppState>,
    Path(incident_id): Path<Uuid>,
    Json(request): Json<AssignReplacementVehicleRequest>,
) -> Result<Json<IncidentResponse>, AppError> {
    // Użyj funkcji SQL
    let success: bool = sqlx::query_scalar(
        "SELECT assign_replacement_vehicle($1, $2)"
    )
    .bind(incident_id)
    .bind(request.replacement_vehicle_id)
    .fetch_one(&state.db)
    .await
    .map_err(|e| {
        if e.to_string().contains("Replacement vehicle is not available") {
            AppError::ValidationError("Pojazd zastępczy nie jest dostępny".to_string())
        } else {
            AppError::DatabaseError(e.to_string())
        }
    })?;

    if !success {
        return Err(AppError::InternalError(
            "Nie udało się przypisać pojazdu zastępczego".to_string()
        ));
    }

    // Pobierz zaktualizowany incydent
    let incident = get_incident_by_id(&state, incident_id).await?;

    Ok(Json(incident))
}

/// GET /incidents/active - aktywne incydenty
pub async fn get_active_incidents(
    State(state): State<AppState>,
) -> Result<Json<IncidentsListResponse>, AppError> {
    let rows = sqlx::query(
        r#"
        SELECT 
            i.id, i.vehicle_id, i.driver_id, i.incident_type as "incident_type: _",
            i.severity as "severity: _", i.title, i.description,
            ST_AsText(i.location) as location, i.reported_at, i.resolved_at,
            i.resolved_by, i.resolution_notes, i.status as "status: _",
            i.replacement_vehicle_id, i.estimated_resolution, i.created_at, i.updated_at,
            v.registration_number, v.brand as vehicle_brand, v.model as vehicle_model,
            v.current_route_id, r.name as route_name, r.number as route_number, r.color as route_color,
            CONCAT(u.first_name, ' ', u.last_name) as driver_name,
            ru.first_name as resolved_by_first_name, ru.last_name as resolved_by_last_name
        FROM incidents i
        JOIN vehicles v ON i.vehicle_id = v.id
        JOIN drivers d ON i.driver_id = d.id
        JOIN users u ON d.user_id = u.id
        LEFT JOIN routes r ON v.current_route_id = r.id
        LEFT JOIN users ru ON i.resolved_by = ru.id
        WHERE i.status IN ('reported', 'in_progress')
        ORDER BY 
            CASE i.severity 
                WHEN 'critical' THEN 1 
                WHEN 'high' THEN 2 
                WHEN 'medium' THEN 3 
                ELSE 4 
            END,
            i.reported_at DESC
        "#
    )
    .fetch_all(&state.db)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    let incidents: Vec<IncidentResponse> = rows.into_iter()
        .map(|row| row_to_incident_response(row))
        .collect();

    let total = incidents.len() as i64;

    Ok(Json(IncidentsListResponse {
        incidents,
        total,
    }))
}

/// GET /incidents/stats - statystyki incydentów
pub async fn get_incident_stats(
    State(state): State<AppState>,
) -> Result<Json<IncidentStats>, AppError> {
    // Pobierz podstawowe statystyki
    let row = sqlx::query(
        r#"
        SELECT 
            COUNT(*) as total,
            COUNT(*) FILTER (WHERE status IN ('reported', 'in_progress')) as active,
            COUNT(*) FILTER (WHERE status = 'resolved' AND resolved_at >= CURRENT_DATE) as resolved_today,
            AVG(
                CASE 
                    WHEN resolved_at IS NOT NULL THEN 
                        EXTRACT(EPOCH FROM (resolved_at - reported_at)) / 60
                    ELSE NULL
                END
            ) as avg_resolution_minutes
        FROM incidents
        "#
    )
    .fetch_one(&state.db)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    let total_incidents: i64 = row.get("total");
    let active_incidents: i64 = row.get("active");
    let resolved_today: i64 = row.get("resolved_today");
    let average_resolution_minutes: Option<f64> = row.get("avg_resolution_minutes");

    // Pobierz statystyki według ważności
    let severity_rows = sqlx::query(
        r#"
        SELECT severity as "severity: Severity", COUNT(*) as count
        FROM incidents
        GROUP BY severity
        "#
    )
    .fetch_all(&state.db)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    let by_severity: Vec<SeverityCount> = severity_rows.into_iter()
        .map(|r| SeverityCount {
            severity: r.get("severity"),
            count: r.get("count"),
        })
        .collect();

    // Pobierz statystyki według typu
    let type_rows = sqlx::query(
        r#"
        SELECT incident_type as "incident_type: IncidentType", COUNT(*) as count
        FROM incidents
        GROUP BY incident_type
        "#
    )
    .fetch_all(&state.db)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    let by_type: Vec<TypeCount> = type_rows.into_iter()
        .map(|r| TypeCount {
            incident_type: r.get("incident_type"),
            count: r.get("count"),
        })
        .collect();

    Ok(Json(IncidentStats {
        total_incidents,
        active_incidents,
        resolved_today,
        by_severity,
        by_type,
        average_resolution_minutes,
    }))
}

/// GET /incidents/:id/notifications - powiadomienia o incydencie
pub async fn get_incident_notifications(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<IncidentNotification>>, AppError> {
    let notifications = sqlx::query_as::<_, IncidentNotification>(
        r#"
        SELECT 
            n.id, n.incident_id, n.route_id, n.message_pl, n.message_en,
            n.sent_at, n.affected_users_count, n.extra_data
        FROM incident_notifications n
        WHERE n.incident_id = $1
        ORDER BY n.sent_at DESC
        "#
    )
    .bind(id)
    .fetch_all(&state.db)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    Ok(Json(notifications))
}

// Helper functions

async fn get_incident_by_id(state: &AppState, id: Uuid) -> Result<IncidentResponse, AppError> {
    let row = sqlx::query(
        r#"
        SELECT 
            i.id, i.vehicle_id, i.driver_id, i.incident_type as "incident_type: _",
            i.severity as "severity: _", i.title, i.description,
            ST_AsText(i.location) as location, i.reported_at, i.resolved_at,
            i.resolved_by, i.resolution_notes, i.status as "status: _",
            i.replacement_vehicle_id, i.estimated_resolution, i.created_at, i.updated_at,
            v.registration_number, v.brand as vehicle_brand, v.model as vehicle_model,
            v.current_route_id, r.name as route_name, r.number as route_number, r.color as route_color,
            CONCAT(u.first_name, ' ', u.last_name) as driver_name,
            ru.first_name as resolved_by_first_name, ru.last_name as resolved_by_last_name
        FROM incidents i
        JOIN vehicles v ON i.vehicle_id = v.id
        JOIN drivers d ON i.driver_id = d.id
        JOIN users u ON d.user_id = u.id
        LEFT JOIN routes r ON v.current_route_id = r.id
        LEFT JOIN users ru ON i.resolved_by = ru.id
        WHERE i.id = $1
        "#
    )
    .bind(id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?
    .ok_or_else(|| AppError::NotFound(format!("Incydent o ID {} nie istnieje", id)))?;

    Ok(row_to_incident_response(row))
}

fn row_to_incident_response(row: sqlx::postgres::PgRow) -> IncidentResponse {
    use sqlx::Row;

    let location: Option<String> = row.get("location");
    let incident_location = location.and_then(|loc| {
        let coords: Vec<&str> = loc
            .trim_start_matches("SRID=4326;POINT(")
            .trim_end_matches(')')
            .split_whitespace()
            .collect();
        if coords.len() == 2 {
            coords[0].parse::<f64>().ok().and_then(|lon| {
                coords[1].parse::<f64>().ok().map(|lat| {
                    crate::models::incident::IncidentLocation { latitude: lat, longitude: lon }
                })
            })
        } else {
            None
        }
    });

    let reported_at: chrono::DateTime<Utc> = row.get("reported_at");
    let resolved_at: Option<chrono::DateTime<Utc>> = row.get("resolved_at");
    let duration_minutes = calculate_duration_minutes(reported_at, resolved_at);

    let resolved_by_name: Option<String> = row.get::<Option<String>, _>("resolved_by_first_name")
        .and_then(|first| {
            row.get::<Option<String>, _>("resolved_by_last_name")
                .map(|last| format!("{} {}", first, last))
        });

    IncidentResponse {
        id: row.get("id"),
        vehicle_id: row.get("vehicle_id"),
        vehicle_info: crate::models::incident::IncidentVehicleInfo {
            id: row.get("vehicle_id"),
            registration_number: row.get("registration_number"),
            brand: row.get("vehicle_brand"),
            model: row.get("vehicle_model"),
            current_route_id: row.get("current_route_id"),
            current_route_name: row.get("route_name"),
        },
        driver_id: row.get("driver_id"),
        driver_name: row.get("driver_name"),
        incident_type: row.get("incident_type"),
        incident_type_name_pl: row.get::<IncidentType, _>("incident_type").to_polish().to_string(),
        severity: row.get("severity"),
        severity_name_pl: row.get::<Severity, _>("severity").to_polish().to_string(),
        severity_color: row.get::<Severity, _>("severity").color().to_string(),
        title: row.get("title"),
        description: row.get("description"),
        location: incident_location,
        reported_at,
        resolved_at,
        resolved_by_name,
        resolution_notes: row.get("resolution_notes"),
        status: row.get("status"),
        status_name_pl: row.get::<IncidentStatus, _>("status").to_polish().to_string(),
        is_active: row.get::<IncidentStatus, _>("status").is_active(),
        replacement_vehicle: None, // TODO: Pobrać dane pojazdu zastępczego
        estimated_resolution: row.get("estimated_resolution"),
        affected_routes: vec![], // TODO: Pobrać dotknięte trasy
        duration_minutes,
    }
}
