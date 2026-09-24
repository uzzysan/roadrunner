use roadrunner::tenant::{scope_request_context, TenantContext, TenantPool};
use sqlx::{postgres::PgPoolOptions, PgPool, Postgres, Transaction};
use uuid::Uuid;

const CARRIER_A: Uuid = Uuid::from_u128(0x00000000000040008000000000000001);
const CARRIER_B: Uuid = Uuid::from_u128(0x00000000000040008000000000000002);
const RLS_TEST_ROLE: &str = "roadrunner_rls_test";
const TENANT_TABLE_COUNT: i64 = 20;
const TENANT_TABLES: [&str; 20] = [
    "carrier_memberships",
    "students",
    "parent_student_links",
    "stops",
    "routes",
    "route_stops",
    "vehicles",
    "drivers",
    "vehicle_locations",
    "tickets",
    "ticket_validations",
    "payments",
    "schedules",
    "incidents",
    "incident_notifications",
    "incident_routes",
    "child_registrations",
    "child_attendance",
    "parent_notifications",
    "ticket_validation_attempts",
];

async fn test_pool() -> Option<PgPool> {
    let database_url = match std::env::var("TEST_DATABASE_URL") {
        Ok(value) => value,
        Err(_) => {
            eprintln!("TEST_DATABASE_URL not set; skipping PostgreSQL RLS test");
            return None;
        }
    };

    Some(
        PgPoolOptions::new()
            .max_connections(5)
            .connect(&database_url)
            .await
            .expect("TEST_DATABASE_URL must point to a disposable PostgreSQL database"),
    )
}

async fn prepare_rls_role(pool: &PgPool) {
    sqlx::query(
        r#"
        DO $do$
        BEGIN
            IF NOT EXISTS (SELECT FROM pg_roles WHERE rolname = 'roadrunner_rls_test') THEN
                CREATE ROLE roadrunner_rls_test NOLOGIN NOBYPASSRLS;
            END IF;
        END
        $do$
        "#,
    )
    .execute(pool)
    .await
    .expect("test database owner must be able to create the RLS test role");

    sqlx::query("GRANT USAGE ON SCHEMA public, app TO roadrunner_rls_test")
        .execute(pool)
        .await
        .expect("RLS test role needs schema access");
    sqlx::query(
        "GRANT SELECT, INSERT, UPDATE, DELETE ON ALL TABLES IN SCHEMA public \
         TO roadrunner_rls_test",
    )
    .execute(pool)
    .await
    .expect("RLS test role needs CRUD grants");
}

async fn assume_rls_role(tx: &mut Transaction<'_, Postgres>) {
    sqlx::query(&format!("SET LOCAL ROLE {RLS_TEST_ROLE}"))
        .execute(&mut **tx)
        .await
        .expect("test transaction must assume the non-bypass RLS role");
}

async fn scoped_transaction<'a>(
    pool: &'a PgPool,
    context: TenantContext,
) -> Transaction<'a, Postgres> {
    let mut tx = pool.begin().await.expect("test transaction must start");
    assume_rls_role(&mut tx).await;
    context
        .apply(&mut tx)
        .await
        .expect("tenant context must be applied");
    tx
}

async fn insert_route(
    tx: &mut Transaction<'_, Postgres>,
    id: Uuid,
    number: &str,
    carrier_id: Option<Uuid>,
) -> Result<u64, sqlx::Error> {
    let result = if let Some(carrier_id) = carrier_id {
        sqlx::query(
            "INSERT INTO routes (id, carrier_id, name, number, description) \
             VALUES ($1, $2, 'RLS test route', $3, 'tenant isolation test')",
        )
        .bind(id)
        .bind(carrier_id)
        .bind(number)
        .execute(&mut **tx)
        .await?
    } else {
        sqlx::query(
            "INSERT INTO routes (id, name, number, description) \
             VALUES ($1, 'RLS test route', $2, 'tenant isolation test')",
        )
        .bind(id)
        .bind(number)
        .execute(&mut **tx)
        .await?
    };

    Ok(result.rows_affected())
}

#[tokio::test]
async fn carrier_rls_isolates_all_crud_and_allows_system_admin() {
    let Some(pool) = test_pool().await else {
        return;
    };
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("test database migrations must succeed");
    prepare_rls_role(&pool).await;

    let tenant_tables: Vec<String> = TENANT_TABLES
        .iter()
        .map(|table| (*table).to_owned())
        .collect();

    let tenant_columns: i64 = sqlx::query_scalar(
        r#"
        SELECT count(*)
        FROM information_schema.columns
        WHERE table_schema = 'public'
          AND column_name = 'carrier_id'
          AND is_nullable = 'NO'
          AND table_name = ANY($1)
        "#,
    )
    .bind(&tenant_tables)
    .fetch_one(&pool)
    .await
    .expect("tenant columns must be inspectable");
    assert_eq!(tenant_columns, TENANT_TABLE_COUNT);

    let protected_tables: i64 = sqlx::query_scalar(
        r#"
        SELECT count(*)
        FROM pg_class c
        JOIN pg_namespace n ON n.oid = c.relnamespace
        WHERE n.nspname = 'public'
          AND c.relkind = 'r'
          AND c.relrowsecurity
          AND c.relforcerowsecurity
          AND c.relname = ANY($1)
        "#,
    )
    .bind(&tenant_tables)
    .fetch_one(&pool)
    .await
    .expect("RLS metadata must be inspectable");
    assert_eq!(protected_tables, TENANT_TABLE_COUNT);

    let crud_policies: i64 = sqlx::query_scalar(
        r#"
        SELECT count(*)
        FROM pg_policies
        WHERE schemaname = 'public'
          AND tablename = ANY($1)
          AND policyname = ANY($2)
        "#,
    )
    .bind(&tenant_tables)
    .bind(vec![
        "carrier_select",
        "carrier_insert",
        "carrier_update",
        "carrier_delete",
    ])
    .fetch_one(&pool)
    .await
    .expect("RLS policies must be inspectable");
    assert_eq!(crud_policies, TENANT_TABLE_COUNT * 4);

    let invoker_views: i64 = sqlx::query_scalar(
        r#"
        SELECT count(*)
        FROM pg_class c
        JOIN pg_namespace n ON n.oid = c.relnamespace
        WHERE n.nspname = 'public'
          AND c.relkind = 'v'
          AND 'security_invoker=true' = ANY(COALESCE(c.reloptions, ARRAY[]::TEXT[]))
        "#,
    )
    .fetch_one(&pool)
    .await
    .expect("view security options must be inspectable");
    assert_eq!(invoker_views, 10);

    let route_a = Uuid::new_v4();
    let route_b = Uuid::new_v4();
    let route_admin = Uuid::new_v4();
    let nonce = Uuid::new_v4().simple().to_string()[..12].to_owned();

    let mut tenant_a = scoped_transaction(&pool, TenantContext::carrier(CARRIER_A)).await;
    assert_eq!(
        insert_route(&mut tenant_a, route_a, &format!("A-{nonce}"), None)
            .await
            .expect("carrier A may create its own route"),
        1
    );
    let stored_carrier: Uuid = sqlx::query_scalar("SELECT carrier_id FROM routes WHERE id = $1")
        .bind(route_a)
        .fetch_one(&mut *tenant_a)
        .await
        .expect("carrier A may read its own route");
    assert_eq!(stored_carrier, CARRIER_A);
    tenant_a
        .commit()
        .await
        .expect("carrier A insert must commit");

    let mut tenant_b = scoped_transaction(&pool, TenantContext::carrier(CARRIER_B)).await;
    let visible_to_b: i64 = sqlx::query_scalar("SELECT count(*) FROM routes WHERE id = $1")
        .bind(route_a)
        .fetch_one(&mut *tenant_b)
        .await
        .expect("carrier B query must succeed without leaking rows");
    assert_eq!(visible_to_b, 0);
    let updated_by_b = sqlx::query("UPDATE routes SET name = 'leaked' WHERE id = $1")
        .bind(route_a)
        .execute(&mut *tenant_b)
        .await
        .expect("cross-tenant update must be filtered")
        .rows_affected();
    assert_eq!(updated_by_b, 0);
    let deleted_by_b = sqlx::query("DELETE FROM routes WHERE id = $1")
        .bind(route_a)
        .execute(&mut *tenant_b)
        .await
        .expect("cross-tenant delete must be filtered")
        .rows_affected();
    assert_eq!(deleted_by_b, 0);
    tenant_b
        .commit()
        .await
        .expect("filtered carrier B transaction must commit");

    let mut wrong_insert = scoped_transaction(&pool, TenantContext::carrier(CARRIER_B)).await;
    assert!(
        insert_route(
            &mut wrong_insert,
            Uuid::new_v4(),
            &format!("WRONG-{nonce}"),
            Some(CARRIER_A),
        )
        .await
        .is_err(),
        "carrier B must not insert a row owned by carrier A"
    );
    wrong_insert
        .rollback()
        .await
        .expect("rejected insert transaction must roll back");

    let mut tenant_b = scoped_transaction(&pool, TenantContext::carrier(CARRIER_B)).await;
    insert_route(&mut tenant_b, route_b, &format!("B-{nonce}"), None)
        .await
        .expect("carrier B may create its own route");
    tenant_b
        .commit()
        .await
        .expect("carrier B insert must commit");

    let stop_a = Uuid::new_v4();
    let stop_b = Uuid::new_v4();
    let mut tenant_a = scoped_transaction(&pool, TenantContext::carrier(CARRIER_A)).await;
    sqlx::query(
        "INSERT INTO stops (id, name, location) \
         VALUES ($1, 'Carrier A stop', ST_GeogFromText('SRID=4326;POINT(21 52)'))",
    )
    .bind(stop_a)
    .execute(&mut *tenant_a)
    .await
    .expect("carrier A stop must be inserted");
    tenant_a.commit().await.expect("carrier A stop must commit");

    let mut tenant_b = scoped_transaction(&pool, TenantContext::carrier(CARRIER_B)).await;
    sqlx::query(
        "INSERT INTO stops (id, name, location) \
         VALUES ($1, 'Carrier B stop', ST_GeogFromText('SRID=4326;POINT(21.1 52.1)'))",
    )
    .bind(stop_b)
    .execute(&mut *tenant_b)
    .await
    .expect("carrier B stop must be inserted");
    sqlx::query("INSERT INTO route_stops (route_id, stop_id, stop_order) VALUES ($1, $2, 1)")
        .bind(route_b)
        .bind(stop_b)
        .execute(&mut *tenant_b)
        .await
        .expect("same-carrier relationship must be accepted");
    tenant_b
        .commit()
        .await
        .expect("same-carrier relationship must commit");

    let mut cross_carrier = scoped_transaction(&pool, TenantContext::carrier(CARRIER_B)).await;
    let cross_carrier_result =
        sqlx::query("INSERT INTO route_stops (route_id, stop_id, stop_order) VALUES ($1, $2, 2)")
            .bind(route_b)
            .bind(stop_a)
            .execute(&mut *cross_carrier)
            .await;
    assert!(
        cross_carrier_result.is_err(),
        "composite foreign keys must reject cross-carrier relationships"
    );
    cross_carrier
        .rollback()
        .await
        .expect("rejected cross-carrier relationship must roll back");

    let mut no_context = pool
        .begin()
        .await
        .expect("no-context transaction must start");
    assume_rls_role(&mut no_context).await;
    let visible_without_context: i64 =
        sqlx::query_scalar("SELECT count(*) FROM routes WHERE id = ANY($1)")
            .bind(vec![route_a, route_b])
            .fetch_one(&mut *no_context)
            .await
            .expect("no-context query must fail closed without error");
    assert_eq!(visible_without_context, 0);
    no_context
        .rollback()
        .await
        .expect("no-context transaction must roll back");

    let mut system_admin = scoped_transaction(&pool, TenantContext::system_admin()).await;
    let visible_to_admin: i64 =
        sqlx::query_scalar("SELECT count(*) FROM routes WHERE id = ANY($1)")
            .bind(vec![route_a, route_b])
            .fetch_one(&mut *system_admin)
            .await
            .expect("system admin may read all carriers");
    assert_eq!(visible_to_admin, 2);
    assert_eq!(
        insert_route(
            &mut system_admin,
            route_admin,
            &format!("ADMIN-{nonce}"),
            Some(CARRIER_A),
        )
        .await
        .expect("system admin may insert for any carrier"),
        1
    );
    let admin_updates = sqlx::query("UPDATE routes SET name = 'admin updated' WHERE id = $1")
        .bind(route_b)
        .execute(&mut *system_admin)
        .await
        .expect("system admin may update any carrier")
        .rows_affected();
    assert_eq!(admin_updates, 1);
    let admin_deletes = sqlx::query("DELETE FROM routes WHERE id = $1")
        .bind(route_admin)
        .execute(&mut *system_admin)
        .await
        .expect("system admin may delete any carrier")
        .rows_affected();
    assert_eq!(admin_deletes, 1);
    system_admin
        .commit()
        .await
        .expect("system-admin CRUD must commit");

    let database_url = std::env::var("TEST_DATABASE_URL").expect("test URL was checked above");
    let runtime_pool = PgPoolOptions::new()
        .max_connections(1)
        .after_connect(|connection, _metadata| {
            Box::pin(async move {
                sqlx::query("SET ROLE roadrunner_rls_test")
                    .execute(connection)
                    .await?;
                Ok(())
            })
        })
        .connect(&database_url)
        .await
        .expect("runtime test pool must connect");
    let tenant_pool = TenantPool::new(runtime_pool);

    let unscoped = sqlx::query_scalar::<_, i64>("SELECT count(*) FROM routes")
        .fetch_one(&tenant_pool)
        .await;
    assert!(
        matches!(unscoped, Err(sqlx::Error::Protocol(_))),
        "tenant pool must reject SQL without an explicit context"
    );

    let visible_to_a = scope_request_context(Some(TenantContext::carrier(CARRIER_A)), async {
        sqlx::query_scalar::<_, i64>("SELECT count(*) FROM routes WHERE id = $1")
            .bind(route_a)
            .fetch_one(&tenant_pool)
            .await
    })
    .await
    .expect("tenant pool must execute scoped queries");
    assert_eq!(visible_to_a, 1);

    let duplicate_result = scope_request_context(Some(TenantContext::carrier(CARRIER_A)), async {
        sqlx::query(
            "INSERT INTO routes (id, name, number, description) \
             VALUES ($1, 'duplicate', $2, 'must fail')",
        )
        .bind(route_a)
        .bind(format!("DUP-{nonce}"))
        .execute(&tenant_pool)
        .await
    })
    .await;
    assert!(duplicate_result.is_err(), "database errors must roll back");

    let cancel_route = Uuid::new_v4();
    let cancel_number = format!("CANCEL-{nonce}");
    let cancellation_pool = tenant_pool.clone();
    let (inserted_sender, inserted_receiver) = tokio::sync::oneshot::channel();
    let cancelled = tokio::spawn(scope_request_context(
        Some(TenantContext::carrier(CARRIER_A)),
        async move {
            let mut tx = cancellation_pool.begin().await.expect("transaction starts");
            insert_route(&mut tx, cancel_route, &cancel_number, None)
                .await
                .expect("cancelled transaction inserts before suspension");
            let _ = inserted_sender.send(());
            sqlx::query("SELECT pg_sleep(30)")
                .execute(&mut *tx)
                .await
                .expect("sleep query runs until cancellation");
            tx.commit().await.expect("unreachable after cancellation");
        },
    ));
    inserted_receiver
        .await
        .expect("cancelled transaction must reach the suspension point");
    cancelled.abort();
    let _ = cancelled.await;

    let count_after_cancel = tokio::time::timeout(
        std::time::Duration::from_secs(5),
        scope_request_context(Some(TenantContext::carrier(CARRIER_A)), async {
            sqlx::query_scalar::<_, i64>("SELECT count(*) FROM routes WHERE id = $1")
                .bind(cancel_route)
                .fetch_one(&tenant_pool)
                .await
        }),
    )
    .await
    .expect("the single pooled connection must be reusable after cancellation")
    .expect("post-cancellation query must succeed");
    assert_eq!(count_after_cancel, 0, "cancelled work must roll back");

    let leaked_to_b = scope_request_context(Some(TenantContext::carrier(CARRIER_B)), async {
        sqlx::query_scalar::<_, i64>("SELECT count(*) FROM routes WHERE id = $1")
            .bind(route_a)
            .fetch_one(&tenant_pool)
            .await
    })
    .await
    .expect("reused connection must accept the next carrier context");
    assert_eq!(leaked_to_b, 0, "connection reuse must not leak carrier A");

    let mut tenant_a = scoped_transaction(&pool, TenantContext::carrier(CARRIER_A)).await;
    let updated_by_a = sqlx::query("UPDATE routes SET name = 'carrier A updated' WHERE id = $1")
        .bind(route_a)
        .execute(&mut *tenant_a)
        .await
        .expect("carrier A may update its own route")
        .rows_affected();
    assert_eq!(updated_by_a, 1);
    let deleted_by_a = sqlx::query("DELETE FROM routes WHERE id = $1")
        .bind(route_a)
        .execute(&mut *tenant_a)
        .await
        .expect("carrier A may delete its own route")
        .rows_affected();
    assert_eq!(deleted_by_a, 1);
    tenant_a.commit().await.expect("carrier A CRUD must commit");

    let mut tenant_a = scoped_transaction(&pool, TenantContext::carrier(CARRIER_A)).await;
    sqlx::query("DELETE FROM stops WHERE id = $1")
        .bind(stop_a)
        .execute(&mut *tenant_a)
        .await
        .expect("carrier A may delete its stop");
    tenant_a
        .commit()
        .await
        .expect("carrier A stop cleanup commits");

    let mut tenant_b = scoped_transaction(&pool, TenantContext::carrier(CARRIER_B)).await;
    let deleted_by_b = sqlx::query("DELETE FROM routes WHERE id = $1")
        .bind(route_b)
        .execute(&mut *tenant_b)
        .await
        .expect("carrier B may delete its own route")
        .rows_affected();
    assert_eq!(deleted_by_b, 1);
    sqlx::query("DELETE FROM stops WHERE id = $1")
        .bind(stop_b)
        .execute(&mut *tenant_b)
        .await
        .expect("carrier B may delete its stop");
    tenant_b
        .commit()
        .await
        .expect("carrier B delete must commit");
}
