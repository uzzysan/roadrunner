pub mod auth;
pub mod payments;
pub mod tickets;

// NOTE (2026-08-24 status review): `route.rs`, `schedule.rs`, and `stop.rs` exist in this
// directory (Phase 3 — OpenStreetMap/stops/schedules) but are deliberately NOT declared as
// modules here yet. Wiring them in surfaces ~80 compile errors: they reference `AppError`
// variants that no longer exist (`DatabaseError`, `ValidationError` — the enum was refactored
// since these handlers were written) and `RouteAtStop` is missing `#[derive(sqlx::FromRow)]`.
// So Phase 3 is less complete than docs/PLAN.md's "🔄 in progress" suggests: the data models
// (models/route.rs, models/stop.rs, models/schedule.rs) are solid, but the handler layer needs
// a real pass — updating error variants and adding the missing FromRow derive — before it can
// be wired into main.rs. Left un-declared rather than force-fixed here to keep this session's
// fix scoped to the three previously-identified bugs (this note, mod declarations for
// tickets/payments, the WsState/AppState FromRef, and the Timelike import).
