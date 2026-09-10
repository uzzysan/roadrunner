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
use chrono::{Datelike, Local, NaiveTime, Timelike, Weekday};
use serde::{Deserialize, Serialize};
use sqlx::QueryBuilder;
use uuid::Uuid;

use crate::{errors::AppError, models::schedule::DayType, state::AppState};

const MAX_NEXT_LIMIT: i32 = 50;

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
#[derive(Debug, Serialize, sqlx::FromRow)]
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

fn parse_query_time(value: &str) -> Result<NaiveTime, AppError> {
    NaiveTime::parse_from_str(value, "%H:%M:%S")
        .or_else(|_| NaiveTime::parse_from_str(value, "%H:%M"))
        .map_err(|_| AppError::ValidationError(format!("Invalid time '{value}'")))
}

fn parse_next_limit(limit: i32) -> Result<i32, AppError> {
    if (1..=MAX_NEXT_LIMIT).contains(&limit) {
        Ok(limit)
    } else {
        Err(AppError::ValidationError(format!(
            "limit must be between 1 and {MAX_NEXT_LIMIT}"
        )))
    }
}

fn format_clock(time: NaiveTime) -> String {
    time.format("%H:%M:%S").to_string()
}

/// GET /schedules - lista rozkładów z filtrowaniem
pub async fn list_schedules(
    State(state): State<AppState>,
    Query(query): Query<SchedulesQuery>,
) -> Result<Json<SchedulesListResponse>, AppError> {
    let from_time = query
        .from_time
        .as_deref()
        .map(parse_query_time)
        .transpose()?;
    let to_time = query.to_time.as_deref().map(parse_query_time).transpose()?;

    let mut qb = QueryBuilder::<sqlx::Postgres>::new(
        r#"
        SELECT
            s.id,
            s.route_id,
            s.stop_id,
            s.arrival_time::text as arrival_time,
            s.departure_time::text as departure_time,
            s.day_type,
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
        "#,
    );

    if let Some(stop_id) = query.stop_id {
        qb.push(" AND s.stop_id = ");
        qb.push_bind(stop_id);
    }

    if let Some(route_id) = query.route_id {
        qb.push(" AND s.route_id = ");
        qb.push_bind(route_id);
    }

    if let Some(day_type) = query.day_type {
        qb.push(" AND s.day_type = ");
        qb.push_bind(day_type);
    }

    if let Some(from_time) = from_time {
        qb.push(" AND s.departure_time >= ");
        qb.push_bind(from_time);
    }

    if let Some(to_time) = to_time {
        qb.push(" AND s.departure_time <= ");
        qb.push_bind(to_time);
    }

    qb.push(" ORDER BY s.departure_time LIMIT 200");

    let rows = qb
        .build_query_as::<ScheduleWithDetails>()
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
    let limit = parse_next_limit(query.limit)?;
    let now = Local::now();
    let current_time = now.time();

    let day_type = match now.weekday() {
        Weekday::Sat => "saturday",
        Weekday::Sun => "sunday",
        _ => "weekday",
    };

    #[derive(sqlx::FromRow)]
    struct NextDepartureRow {
        schedule_id: Uuid,
        departure_time: NaiveTime,
        route_id: Uuid,
        route_name: String,
        route_number: String,
        route_color: String,
        stop_id: Uuid,
        stop_name: String,
    }

    let mut qb = QueryBuilder::<sqlx::Postgres>::new(
        r#"
        SELECT
            s.id as schedule_id,
            s.departure_time as departure_time,
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
          AND s.departure_time >=
        "#,
    );
    qb.push_bind(current_time);
    qb.push(" AND (s.day_type::text = ");
    qb.push_bind(day_type);
    qb.push(" OR s.day_type = 'everyday')");

    if let Some(stop_id) = query.stop_id {
        qb.push(" AND s.stop_id = ");
        qb.push_bind(stop_id);
    }

    if let Some(route_id) = query.route_id {
        qb.push(" AND s.route_id = ");
        qb.push_bind(route_id);
    }

    qb.push(" ORDER BY s.departure_time LIMIT ");
    qb.push_bind(limit);

    let rows = qb
        .build_query_as::<NextDepartureRow>()
        .fetch_all(&state.db)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    let current_minutes = current_time.hour() as i64 * 60 + current_time.minute() as i64;
    let departures = rows
        .into_iter()
        .map(|row| {
            let departure_minutes =
                row.departure_time.hour() as i64 * 60 + row.departure_time.minute() as i64;

            NextDeparture {
                schedule_id: row.schedule_id,
                departure_time: format_clock(row.departure_time),
                route_id: row.route_id,
                route_name: row.route_name,
                route_number: row.route_number,
                route_color: row.route_color,
                stop_id: row.stop_id,
                stop_name: row.stop_name,
                minutes_until_departure: departure_minutes - current_minutes,
            }
        })
        .collect();

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

    let (day_type, day_name) = match now.weekday() {
        Weekday::Mon => ("weekday", "Poniedziałek"),
        Weekday::Tue => ("weekday", "Wtorek"),
        Weekday::Wed => ("weekday", "Środa"),
        Weekday::Thu => ("weekday", "Czwartek"),
        Weekday::Fri => ("weekday", "Piątek"),
        Weekday::Sat => ("saturday", "Sobota"),
        Weekday::Sun => ("sunday", "Niedziela"),
    };

    #[derive(sqlx::FromRow)]
    struct RouteRow {
        id: Uuid,
        name: String,
        number: String,
        color: String,
    }

    let routes = sqlx::query_as::<_, RouteRow>(
        r#"
        SELECT id, name, number, color
        FROM routes
        WHERE is_active = true
        ORDER BY
            CASE
                WHEN number ~ '^[0-9]+$' THEN number::int
                ELSE 999999
            END
        "#,
    )
    .fetch_all(&state.db)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    let mut departures_by_route = Vec::new();

    #[derive(sqlx::FromRow)]
    struct DepartureRow {
        schedule_id: Uuid,
        stop_id: Uuid,
        stop_name: String,
        departure_time: NaiveTime,
    }

    for route in routes {
        let departures = sqlx::query_as::<_, DepartureRow>(
            r#"
            SELECT
                s.id as schedule_id,
                s.stop_id,
                st.name as stop_name,
                s.departure_time as departure_time
            FROM schedules s
            JOIN stops st ON s.stop_id = st.id
            WHERE s.route_id = $1
              AND s.is_active = true
              AND st.is_active = true
              AND (s.day_type::text = $2 OR s.day_type = 'everyday')
            ORDER BY s.departure_time
            "#,
        )
        .bind(route.id)
        .bind(day_type)
        .fetch_all(&state.db)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        if !departures.is_empty() {
            let today_departures: Vec<TodayDeparture> = departures
                .into_iter()
                .map(|d| TodayDeparture {
                    schedule_id: d.schedule_id,
                    stop_id: d.stop_id,
                    stop_name: d.stop_name,
                    departure_time: format_clock(d.departure_time),
                    is_past: d.departure_time < current_time,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_query_time_accepts_hhmmss_and_hhmm() {
        assert_eq!(
            parse_query_time("08:30:00").unwrap(),
            NaiveTime::from_hms_opt(8, 30, 0).unwrap()
        );
        assert_eq!(
            parse_query_time("08:30").unwrap(),
            NaiveTime::from_hms_opt(8, 30, 0).unwrap()
        );
    }

    #[test]
    fn parse_query_time_rejects_injection_and_garbage() {
        assert!(parse_query_time("nope").is_err());
        assert!(parse_query_time("00:00' OR 1=1 --").is_err());
    }

    #[test]
    fn parse_next_limit_accepts_range() {
        assert_eq!(parse_next_limit(1).unwrap(), 1);
        assert_eq!(parse_next_limit(5).unwrap(), 5);
        assert_eq!(parse_next_limit(50).unwrap(), 50);
    }

    #[test]
    fn parse_next_limit_rejects_out_of_range() {
        assert!(parse_next_limit(0).is_err());
        assert!(parse_next_limit(-1).is_err());
        assert!(parse_next_limit(51).is_err());
    }
}
