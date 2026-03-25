//! Handler dla rozkładów jazdy (schedules)
//!
//! Endpointy:
//! - GET /schedules - lista rozkładów z filtrowaniem
//! - GET /schedules/next?stop_id={}&route_id={} - najbliższe odjazdy
//! - GET /schedules/today - dzisiejsze odjazdy

use axum::{
    extract::{Query, State},
    Json,
};
use chrono::{Local, NaiveTime, Weekday};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    models::schedule::{Schedule, ScheduleWithRoute, ScheduleWithStop, DayType},
    state::AppState,
    errors::AppError,
};

/// Query parameters dla filtrowania rozkładów
#[derive(Debug, Deserialize)]
pub struct SchedulesQuery {
    /// Filtrowanie po ID przystanku
    pub stop_id: Option<Uuid>,
    /// Filtrowanie po ID trasy
    pub route_id: Option<Uuid>,
    /// Filtrowanie po typie dnia
    pub day_type: Option<DayType>,
    /// Filtrowanie po godzinie od
    pub from_time: Option<String>,
    /// Filtrowanie po godzinie do
    pub to_time: Option<String>,
}

/// Response dla listy rozkładów
#[derive(Debug, Serialize)]
pub struct SchedulesListResponse {
    pub schedules: Vec<ScheduleWithDetails>,
    pub total: i64,
}

/// Szczegóły rozkładu z powiązanymi danymi
#[derive(Debug, Serialize)]
pub struct ScheduleWithDetails {
    pub id: Uuid,
    pub route_id: Uuid,
    pub stop_id: Uuid,
    pub arrival_time: String,
    pub departure_time: String,
    pub day_type: DayType,
    pub is_active: bool,
    pub route_name: String,
    pub route_number: String,
    pub route_color: String,
    pub stop_name: String,
    pub stop_latitude: f64,
    pub stop_longitude: f64,
}

/// GET /schedules - lista rozkładów z filtrowaniem
pub async fn list_schedules(
    State(state): State<AppState>,
    Query(query): Query<SchedulesQuery>,
) -> Result<Json<SchedulesListResponse>, AppError> {
    let mut sql = String::from(
        r#"
        SELECT 
            s.id,
            s.route_id,
            s.stop_id,
            s.arrival_time::text as arrival_time,
            s.departure_time::text as departure_time,
            s.day_type as "day_type: DayType",
            s.is_active,
            r.name as route_name,
            r.number as route_number,
            r.color as route_color,
            st.name as stop_name,
            ST_Y(st.location::geometry) as stop_latitude,
            ST_X(st.location::geometry) as stop_longitude
        FROM schedules s
        JOIN routes r ON s.route_id = r.id
        JOIN stops st ON s.stop_id = st.id
        WHERE s.is_active = true
          AND r.is_active = true
          AND st.is_active = true
        "#
    );

    let mut conditions = Vec::new();

    if let Some(stop_id) = query.stop_id {
        conditions.push(format!("AND s.stop_id = '{}'", stop_id));
    }

    if let Some(route_id) = query.route_id {
        conditions.push(format!("AND s.route_id = '{}'", route_id));
    }

    if let Some(day_type) = query.day_type {
        conditions.push(format!("AND s.day_type = '{:?}'", day_type).to_lowercase());
    }

    if let Some(from_time) = query.from_time {
        conditions.push(format!("AND s.departure_time >= '{}'", from_time));
    }

    if let Some(to_time) = query.to_time {
        conditions.push(format!("AND s.departure_time <= '{}'", to_time));
    }

    for condition in conditions {
        sql.push_str(&condition);
    }

    sql.push_str(" ORDER BY s.departure_time LIMIT 200");

    let rows = sqlx::query_as::<_, ScheduleWithDetails>(&sql)
        .fetch_all(&state.db)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    let total = rows.len() as i64;

    Ok(Json(SchedulesListResponse {
        schedules: rows,
        total,
    }))
}

/// Query parameters dla najbliższych odjazdów
#[derive(Debug, Deserialize)]
pub struct NextDeparturesQuery {
    /// ID przystanku (opcjonalne - jeśli nie podane, szuka we wszystkich)
    pub stop_id: Option<Uuid>,
    /// ID linii (opcjonalne)
    pub route_id: Option<Uuid>,
    /// Liczba odjazdów do zwrócenia (domyślnie 5)
    #[serde(default = "default_limit")]
    pub limit: i32,
}

fn default_limit() -> i32 {
    5
}

/// Response dla najbliższych odjazdów
#[derive(Debug, Serialize)]
pub struct NextDeparture {
    pub schedule_id: Uuid,
    pub departure_time: String,
    pub route_id: Uuid,
    pub route_name: String,
    pub route_number: String,
    pub route_color: String,
    pub stop_id: Uuid,
    pub stop_name: String,
    /// Czas do odjazdu w minutach
    pub minutes_until_departure: i64,
}

/// GET /schedules/next - najbliższe odjazdy
/// 
/// Zwraca najbliższe odjazdy od aktualnej godziny
pub async fn next_departures(
    State(state): State<AppState>,
    Query(query): Query<NextDeparturesQuery>,
) -> Result<Json<Vec<NextDeparture>>, AppError> {
    let now = Local::now();
    let current_time = now.time();
    let current_time_str = current_time.format("%H:%M:%S").to_string();
    
    // Określ typ dnia na podstawie dzisiejszego dnia tygodnia
    let day_type = match now.weekday() {
        Weekday::Sat => "saturday",
        Weekday::Sun => "sunday",
        _ => "weekday",
    };

    let mut sql = String::from(
        r#"
        SELECT 
            s.id as schedule_id,
            s.departure_time::text as departure_time,
            s.route_id,
            r.name as route_name,
            r.number as route_number,
            r.color as route_color,
            s.stop_id,
            st.name as stop_name
        FROM schedules s
        JOIN routes r ON s.route_id = r.id
        JOIN stops st ON s.stop_id = st.id
        WHERE s.is_active = true
          AND r.is_active = true
          AND st.is_active = true
          AND s.departure_time >= $1
          AND (s.day_type = $2 OR s.day_type = 'everyday')
        "#
    );

    if let Some(stop_id) = query.stop_id {
        sql.push_str(&format!(" AND s.stop_id = '{}'", stop_id));
    }

    if let Some(route_id) = query.route_id {
        sql.push_str(&format!(" AND s.route_id = '{}'", route_id));
    }

    sql.push_str(" ORDER BY s.departure_time LIMIT $3");

    let rows = sqlx::query(&sql)
        .bind(&current_time_str)
        .bind(day_type)
        .bind(query.limit)
        .fetch_all(&state.db)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    let mut departures = Vec::new();

    for row in rows {
        let departure_time_str: String = row.try_get("departure_time")
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        
        let departure_time = NaiveTime::parse_from_str(&departure_time_str, "%H:%M:%S")
            .map_err(|e| AppError::InternalError(format!("Błąd parsowania czasu: {}", e)))?;

        // Oblicz minuty do odjazdu
        let current_minutes = current_time.hour() as i64 * 60 + current_time.minute() as i64;
        let departure_minutes = departure_time.hour() as i64 * 60 + departure_time.minute() as i64;
        let minutes_until = departure_minutes - current_minutes;

        departures.push(NextDeparture {
            schedule_id: row.try_get("schedule_id")
                .map_err(|e| AppError::DatabaseError(e.to_string()))?,
            departure_time: departure_time_str,
            route_id: row.try_get("route_id")
                .map_err(|e| AppError::DatabaseError(e.to_string()))?,
            route_name: row.try_get("route_name")
                .map_err(|e| AppError::DatabaseError(e.to_string()))?,
            route_number: row.try_get("route_number")
                .map_err(|e| AppError::DatabaseError(e.to_string()))?,
            route_color: row.try_get("route_color")
                .map_err(|e| AppError::DatabaseError(e.to_string()))?,
            stop_id: row.try_get("stop_id")
                .map_err(|e| AppError::DatabaseError(e.to_string()))?,
            stop_name: row.try_get("stop_name")
                .map_err(|e| AppError::DatabaseError(e.to_string()))?,
            minutes_until_departure: minutes_until,
        });
    }

    Ok(Json(departures))
}

/// Response dla dzisiejszych odjazdów pogrupowanych
#[derive(Debug, Serialize)]
pub struct TodaySchedulesResponse {
    pub date: String,
    pub day_type: String,
    pub day_name: String,
    pub departures_by_route: Vec<RouteTodayDepartures>,
}

#[derive(Debug, Serialize)]
pub struct RouteTodayDepartures {
    pub route_id: Uuid,
    pub route_name: String,
    pub route_number: String,
    pub route_color: String,
    pub departures: Vec<TodayDeparture>,
}

#[derive(Debug, Serialize)]
pub struct TodayDeparture {
    pub schedule_id: Uuid,
    pub stop_id: Uuid,
    pub stop_name: String,
    pub departure_time: String,
    pub is_past: bool,
}

/// GET /schedules/today - dzisiejsze odjazdy pogrupowane według linii
pub async fn today_schedules(
    State(state): State<AppState>,
) -> Result<Json<TodaySchedulesResponse>, AppError> {
    let now = Local::now();
    let current_time = now.time();
    let current_time_str = current_time.format("%H:%M:%S").to_string();
    
    // Określ typ dnia i nazwę
    let (day_type, day_name) = match now.weekday() {
        Weekday::Mon => ("weekday", "Poniedziałek"),
        Weekday::Tue => ("weekday", "Wtorek"),
        Weekday::Wed => ("weekday", "Środa"),
        Weekday::Thu => ("weekday", "Czwartek"),
        Weekday::Fri => ("weekday", "Piątek"),
        Weekday::Sat => ("saturday", "Sobota"),
        Weekday::Sun => ("sunday", "Niedziela"),
    };

    // Pobierz wszystkie aktywne linie
    let routes = sqlx::query!(
        r#"
        SELECT id, name, number, color
        FROM routes
        WHERE is_active = true
        ORDER BY 
            CASE 
                WHEN number ~ '^[0-9]+$' THEN number::int
                ELSE 999999
            END
        "#
    )
    .fetch_all(&state.db)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    let mut departures_by_route = Vec::new();

    for route in routes {
        // Pobierz dzisiejsze odjazdy dla tej linii
        let departures = sqlx::query!(
            r#"
            SELECT 
                s.id as schedule_id,
                s.stop_id,
                st.name as stop_name,
                s.departure_time::text as departure_time
            FROM schedules s
            JOIN stops st ON s.stop_id = st.id
            WHERE s.route_id = $1
              AND s.is_active = true
              AND st.is_active = true
              AND (s.day_type = $2 OR s.day_type = 'everyday')
            ORDER BY s.departure_time
            "#,
            route.id,
            day_type
        )
        .fetch_all(&state.db)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        if !departures.is_empty() {
            let today_departures: Vec<TodayDeparture> = departures
                .into_iter()
                .map(|d| {
                    let is_past = d.departure_time.as_ref()
                        .map(|t| t < &current_time_str)
                        .unwrap_or(false);
                    
                    TodayDeparture {
                        schedule_id: d.schedule_id,
                        stop_id: d.stop_id,
                        stop_name: d.stop_name,
                        departure_time: d.departure_time.unwrap_or_default(),
                        is_past,
                    }
                })
                .collect();

            departures_by_route.push(RouteTodayDepartures {
                route_id: route.id,
                route_name: route.name,
                route_number: route.number,
                route_color: route.color,
                departures: today_departures,
            });
        }
    }

    Ok(Json(TodaySchedulesResponse {
        date: now.format("%Y-%m-%d").to_string(),
        day_type: day_type.to_string(),
        day_name: day_name.to_string(),
        departures_by_route,
    }))
}
