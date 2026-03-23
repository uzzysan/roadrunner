use sqlx::{PgPool, postgres::PgPoolOptions};
use std::env;

pub async fn setup_test_db() -> PgPool {
    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set for tests");
    
    PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to connect to test database")
}

pub async fn cleanup_test_data(pool: &PgPool) {
    sqlx::query("DELETE FROM gps_positions WHERE vehicle_id IN (SELECT id FROM vehicles WHERE registration_number LIKE 'TEST_%')")
        .execute(pool)
        .await
        .ok();
    
    sqlx::query("DELETE FROM vehicle_assignments WHERE vehicle_id IN (SELECT id FROM vehicles WHERE registration_number LIKE 'TEST_%')")
        .execute(pool)
        .await
        .ok();
    
    sqlx::query("DELETE FROM vehicles WHERE registration_number LIKE 'TEST_%'")
        .execute(pool)
        .await
        .ok();
    
    sqlx::query("DELETE FROM users WHERE email LIKE 'test_%@example.com'")
        .execute(pool)
        .await
        .ok();
}
