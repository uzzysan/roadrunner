-- BCK-24: carrier tenancy and PostgreSQL row-level security.
--
-- Global data:
--   * carriers: tenant registry
--   * users: authentication identities (carrier access lives in memberships)
--   * stripe_webhook_events: provider-level idempotency ledger
--   * PostGIS and SQLx metadata
-- Everything listed in tenant_tables below is tenant-owned.

CREATE SCHEMA IF NOT EXISTS app;

-- Runtime connections inherit this NOLOGIN role. Deployment creates a
-- separate LOGIN role and grants it membership; keeping the group role here
-- makes database privileges part of the migration rather than container
-- bootstrap state.
DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'roadrunner_app') THEN
        CREATE ROLE roadrunner_app NOLOGIN NOSUPERUSER NOCREATEDB NOCREATEROLE
            NOINHERIT NOBYPASSRLS;
    END IF;
END
$$;

CREATE OR REPLACE FUNCTION app.current_carrier_id()
RETURNS UUID
LANGUAGE SQL
STABLE
PARALLEL SAFE
AS $$
    SELECT NULLIF(current_setting('app.carrier_id', true), '')::UUID
$$;

CREATE OR REPLACE FUNCTION app.is_system_admin()
RETURNS BOOLEAN
LANGUAGE SQL
STABLE
PARALLEL SAFE
AS $$
    SELECT COALESCE(current_setting('app.system_admin', true), 'false') = 'true'
$$;

GRANT USAGE ON SCHEMA app TO PUBLIC;
GRANT EXECUTE ON FUNCTION app.current_carrier_id() TO PUBLIC;
GRANT EXECUTE ON FUNCTION app.is_system_admin() TO PUBLIC;

CREATE TABLE carriers (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    slug VARCHAR(100) NOT NULL UNIQUE,
    name VARCHAR(255) NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TRIGGER carriers_updated_at
    BEFORE UPDATE ON carriers
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

INSERT INTO carriers (id, slug, name) VALUES
    ('00000000-0000-4000-8000-000000000001', 'test-carrier-a', 'Test Carrier A'),
    ('00000000-0000-4000-8000-000000000002', 'test-carrier-b', 'Test Carrier B');

CREATE TABLE carrier_memberships (
    carrier_id UUID NOT NULL DEFAULT app.current_carrier_id()
        REFERENCES carriers(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (carrier_id, user_id)
);

CREATE OR REPLACE FUNCTION app.can_access_carrier(request_user_id UUID, requested_carrier_id UUID)
RETURNS BOOLEAN
LANGUAGE SQL
STABLE
SECURITY DEFINER
SET search_path = pg_catalog, public
AS $$
    SELECT EXISTS (
        SELECT 1
        FROM carrier_memberships membership
        JOIN carriers carrier ON carrier.id = membership.carrier_id
        WHERE membership.user_id = request_user_id
          AND membership.carrier_id = requested_carrier_id
          AND carrier.is_active
    )
$$;

CREATE OR REPLACE FUNCTION app.is_active_carrier(requested_carrier_id UUID)
RETURNS BOOLEAN
LANGUAGE SQL
STABLE
SECURITY DEFINER
SET search_path = pg_catalog, public
AS $$
    SELECT EXISTS (
        SELECT 1 FROM carriers
        WHERE id = requested_carrier_id AND is_active
    )
$$;

-- Existing single-tenant data belongs to the first seeded carrier. New rows
-- inherit the transaction-local carrier context and fail closed without one.
DO $$
DECLARE
    tenant_table TEXT;
    tenant_tables CONSTANT TEXT[] := ARRAY[
        'students',
        'parent_student_links',
        'stops',
        'routes',
        'route_stops',
        'vehicles',
        'drivers',
        'vehicle_locations',
        'tickets',
        'ticket_validations',
        'payments',
        'schedules',
        'incidents',
        'incident_notifications',
        'incident_routes',
        'child_registrations',
        'child_attendance',
        'parent_notifications',
        'ticket_validation_attempts'
    ];
BEGIN
    FOREACH tenant_table IN ARRAY tenant_tables LOOP
        EXECUTE format('ALTER TABLE %I ADD COLUMN carrier_id UUID', tenant_table);
        EXECUTE format(
            'UPDATE %I SET carrier_id = $1 WHERE carrier_id IS NULL',
            tenant_table
        ) USING '00000000-0000-4000-8000-000000000001'::UUID;
        EXECUTE format(
            'ALTER TABLE %I ALTER COLUMN carrier_id SET DEFAULT app.current_carrier_id()',
            tenant_table
        );
        EXECUTE format(
            'ALTER TABLE %I ALTER COLUMN carrier_id SET NOT NULL',
            tenant_table
        );
        EXECUTE format(
            'ALTER TABLE %I ADD CONSTRAINT %I FOREIGN KEY (carrier_id) REFERENCES carriers(id)',
            tenant_table,
            tenant_table || '_carrier_id_fkey'
        );
        EXECUTE format(
            'CREATE INDEX %I ON %I (carrier_id)',
            tenant_table || '_carrier_id_idx',
            tenant_table
        );
    END LOOP;
END
$$;

-- A composite foreign key is the database-level guarantee that both ends of
-- every tenant-owned relationship belong to the same carrier. UUID primary
-- keys remain globally unique, while these additional keys make
-- (carrier_id, id) valid FK targets.
DO $$
DECLARE
    tenant_table TEXT;
    tenant_tables CONSTANT TEXT[] := ARRAY[
        'students',
        'parent_student_links',
        'stops',
        'routes',
        'route_stops',
        'vehicles',
        'drivers',
        'vehicle_locations',
        'tickets',
        'ticket_validations',
        'payments',
        'schedules',
        'incidents',
        'incident_notifications',
        'incident_routes',
        'child_registrations',
        'child_attendance',
        'parent_notifications',
        'ticket_validation_attempts'
    ];
BEGIN
    FOREACH tenant_table IN ARRAY tenant_tables LOOP
        EXECUTE format(
            'ALTER TABLE %I ADD CONSTRAINT %I UNIQUE (carrier_id, id)',
            tenant_table,
            tenant_table || '_carrier_id_id_key'
        );
    END LOOP;
END
$$;

DO $$
DECLARE
    relation RECORD;
    delete_clause TEXT;
BEGIN
    FOR relation IN
        SELECT * FROM (VALUES
            ('parent_student_links', 'parent_student_links_student_id_fkey', 'student_id', 'students', 'CASCADE'),
            ('route_stops', 'route_stops_route_id_fkey', 'route_id', 'routes', 'CASCADE'),
            ('route_stops', 'route_stops_stop_id_fkey', 'stop_id', 'stops', 'CASCADE'),
            ('schedules', 'schedules_route_id_fkey', 'route_id', 'routes', 'CASCADE'),
            ('schedules', 'schedules_stop_id_fkey', 'stop_id', 'stops', 'CASCADE'),
            ('vehicles', 'vehicles_current_route_id_fkey', 'current_route_id', 'routes', 'SET NULL'),
            ('drivers', 'drivers_assigned_vehicle_id_fkey', 'assigned_vehicle_id', 'vehicles', 'SET NULL'),
            ('vehicle_locations', 'vehicle_locations_vehicle_id_fkey', 'vehicle_id', 'vehicles', 'CASCADE'),
            ('vehicle_locations', 'vehicle_locations_next_stop_id_fkey', 'next_stop_id', 'stops', 'NO ACTION'),
            ('tickets', 'tickets_route_id_fkey', 'route_id', 'routes', 'SET NULL'),
            ('tickets', 'tickets_start_stop_id_fkey', 'start_stop_id', 'stops', 'SET NULL'),
            ('tickets', 'tickets_end_stop_id_fkey', 'end_stop_id', 'stops', 'SET NULL'),
            ('ticket_validations', 'ticket_validations_ticket_id_fkey', 'ticket_id', 'tickets', 'CASCADE'),
            ('ticket_validations', 'ticket_validations_vehicle_id_fkey', 'vehicle_id', 'vehicles', 'SET NULL'),
            ('payments', 'payments_ticket_id_fkey', 'ticket_id', 'tickets', 'SET NULL'),
            ('incidents', 'incidents_vehicle_id_fkey', 'vehicle_id', 'vehicles', 'CASCADE'),
            ('incidents', 'incidents_driver_id_fkey', 'driver_id', 'drivers', 'NO ACTION'),
            ('incidents', 'incidents_replacement_vehicle_id_fkey', 'replacement_vehicle_id', 'vehicles', 'NO ACTION'),
            ('incident_notifications', 'incident_notifications_incident_id_fkey', 'incident_id', 'incidents', 'CASCADE'),
            ('incident_notifications', 'incident_notifications_route_id_fkey', 'route_id', 'routes', 'NO ACTION'),
            ('incident_routes', 'incident_routes_incident_id_fkey', 'incident_id', 'incidents', 'CASCADE'),
            ('incident_routes', 'incident_routes_route_id_fkey', 'route_id', 'routes', 'NO ACTION'),
            ('child_registrations', 'child_registrations_assigned_route_id_fkey', 'assigned_route_id', 'routes', 'NO ACTION'),
            ('child_registrations', 'child_registrations_pickup_stop_id_fkey', 'pickup_stop_id', 'stops', 'NO ACTION'),
            ('child_registrations', 'child_registrations_dropoff_stop_id_fkey', 'dropoff_stop_id', 'stops', 'NO ACTION'),
            ('child_attendance', 'child_attendance_child_id_fkey', 'child_id', 'child_registrations', 'CASCADE'),
            ('child_attendance', 'child_attendance_route_id_fkey', 'route_id', 'routes', 'NO ACTION'),
            ('child_attendance', 'child_attendance_vehicle_id_fkey', 'vehicle_id', 'vehicles', 'NO ACTION'),
            ('child_attendance', 'child_attendance_driver_id_fkey', 'driver_id', 'drivers', 'NO ACTION'),
            ('child_attendance', 'child_attendance_pickup_stop_id_fkey', 'pickup_stop_id', 'stops', 'NO ACTION'),
            ('child_attendance', 'child_attendance_dropoff_stop_id_fkey', 'dropoff_stop_id', 'stops', 'NO ACTION'),
            ('parent_notifications', 'parent_notifications_child_id_fkey', 'child_id', 'child_registrations', 'CASCADE'),
            ('ticket_validation_attempts', 'ticket_validation_attempts_ticket_id_fkey', 'ticket_id', 'tickets', 'NO ACTION'),
            ('ticket_validation_attempts', 'ticket_validation_attempts_vehicle_id_fkey', 'vehicle_id', 'vehicles', 'NO ACTION')
        ) AS relationships(child_table, constraint_name, child_column, parent_table, delete_action)
    LOOP
        delete_clause := CASE relation.delete_action
            WHEN 'SET NULL' THEN format('SET NULL (%I)', relation.child_column)
            ELSE relation.delete_action
        END;
        EXECUTE format(
            'ALTER TABLE %I DROP CONSTRAINT IF EXISTS %I',
            relation.child_table,
            relation.constraint_name
        );
        EXECUTE format(
            'ALTER TABLE %I ADD CONSTRAINT %I FOREIGN KEY (carrier_id, %I) '
            || 'REFERENCES %I (carrier_id, id) ON DELETE %s',
            relation.child_table,
            relation.constraint_name,
            relation.child_column,
            relation.parent_table,
            delete_clause
        );
    END LOOP;
END
$$;

INSERT INTO carrier_memberships (carrier_id, user_id)
SELECT '00000000-0000-4000-8000-000000000001', id
FROM users
ON CONFLICT DO NOTHING;

-- Natural identifiers are unique inside a carrier, not across all carriers.
ALTER TABLE students DROP CONSTRAINT IF EXISTS students_student_id_key;
CREATE UNIQUE INDEX students_carrier_student_id_key
    ON students (carrier_id, student_id)
    WHERE student_id IS NOT NULL;

ALTER TABLE routes DROP CONSTRAINT IF EXISTS routes_number_key;
CREATE UNIQUE INDEX routes_carrier_number_key ON routes (carrier_id, number);

ALTER TABLE vehicles DROP CONSTRAINT IF EXISTS vehicles_registration_number_key;
CREATE UNIQUE INDEX vehicles_carrier_registration_key
    ON vehicles (carrier_id, registration_number);

ALTER TABLE vehicles DROP CONSTRAINT IF EXISTS vehicles_vin_key;
CREATE UNIQUE INDEX vehicles_carrier_vin_key
    ON vehicles (carrier_id, vin)
    WHERE vin IS NOT NULL;

ALTER TABLE drivers DROP CONSTRAINT IF EXISTS drivers_employee_id_key;
CREATE UNIQUE INDEX drivers_carrier_employee_id_key
    ON drivers (carrier_id, employee_id)
    WHERE employee_id IS NOT NULL;

ALTER TABLE tickets DROP CONSTRAINT IF EXISTS tickets_qr_code_key;
CREATE UNIQUE INDEX tickets_carrier_qr_code_key ON tickets (carrier_id, qr_code);

ALTER TABLE child_registrations
    DROP CONSTRAINT IF EXISTS child_registrations_qr_code_key;
CREATE UNIQUE INDEX child_registrations_carrier_qr_code_key
    ON child_registrations (carrier_id, qr_code);

-- Existing views must run with the caller's privileges; otherwise the view
-- owner could bypass RLS on underlying tables.
ALTER VIEW vehicle_driver_view SET (security_invoker = true);
ALTER VIEW driver_vehicle_view SET (security_invoker = true);
ALTER VIEW active_vehicles_view SET (security_invoker = true);
ALTER VIEW active_incidents_view SET (security_invoker = true);
ALTER VIEW incident_stats_view SET (security_invoker = true);
ALTER VIEW incident_notifications_view SET (security_invoker = true);
ALTER VIEW children_view SET (security_invoker = true);
ALTER VIEW today_attendance_view SET (security_invoker = true);
ALTER VIEW child_attendance_history_view SET (security_invoker = true);
ALTER VIEW unread_notifications_view SET (security_invoker = true);

DO $$
DECLARE
    tenant_table TEXT;
    tenant_tables CONSTANT TEXT[] := ARRAY[
        'carrier_memberships',
        'students',
        'parent_student_links',
        'stops',
        'routes',
        'route_stops',
        'vehicles',
        'drivers',
        'vehicle_locations',
        'tickets',
        'ticket_validations',
        'payments',
        'schedules',
        'incidents',
        'incident_notifications',
        'incident_routes',
        'child_registrations',
        'child_attendance',
        'parent_notifications',
        'ticket_validation_attempts'
    ];
BEGIN
    FOREACH tenant_table IN ARRAY tenant_tables LOOP
        EXECUTE format('ALTER TABLE %I ENABLE ROW LEVEL SECURITY', tenant_table);
        EXECUTE format('ALTER TABLE %I FORCE ROW LEVEL SECURITY', tenant_table);

        EXECUTE format(
            'CREATE POLICY carrier_select ON %I FOR SELECT USING '
            || '(app.is_system_admin() OR carrier_id = app.current_carrier_id())',
            tenant_table
        );
        EXECUTE format(
            'CREATE POLICY carrier_insert ON %I FOR INSERT WITH CHECK '
            || '(app.is_system_admin() OR carrier_id = app.current_carrier_id())',
            tenant_table
        );
        EXECUTE format(
            'CREATE POLICY carrier_update ON %I FOR UPDATE USING '
            || '(app.is_system_admin() OR carrier_id = app.current_carrier_id()) '
            || 'WITH CHECK (app.is_system_admin() OR carrier_id = app.current_carrier_id())',
            tenant_table
        );
        EXECUTE format(
            'CREATE POLICY carrier_delete ON %I FOR DELETE USING '
            || '(app.is_system_admin() OR carrier_id = app.current_carrier_id())',
            tenant_table
        );
    END LOOP;
END
$$;

GRANT USAGE ON SCHEMA public, app TO roadrunner_app;
GRANT SELECT, INSERT, UPDATE, DELETE ON ALL TABLES IN SCHEMA public TO roadrunner_app;
GRANT USAGE, SELECT ON ALL SEQUENCES IN SCHEMA public TO roadrunner_app;
GRANT EXECUTE ON FUNCTION app.current_carrier_id() TO roadrunner_app;
GRANT EXECUTE ON FUNCTION app.is_system_admin() TO roadrunner_app;
GRANT EXECUTE ON FUNCTION app.can_access_carrier(UUID, UUID) TO roadrunner_app;
GRANT EXECUTE ON FUNCTION app.is_active_carrier(UUID) TO roadrunner_app;

ALTER DEFAULT PRIVILEGES IN SCHEMA public
    GRANT SELECT, INSERT, UPDATE, DELETE ON TABLES TO roadrunner_app;
ALTER DEFAULT PRIVILEGES IN SCHEMA public
    GRANT USAGE, SELECT ON SEQUENCES TO roadrunner_app;

COMMENT ON TABLE carriers IS 'Global registry of RoadRunner transport operators';
COMMENT ON TABLE carrier_memberships IS
    'Tenant-scoped link between a global login identity and a carrier';
COMMENT ON FUNCTION app.current_carrier_id() IS
    'Returns the carrier selected for the current transaction, or NULL when unset';
COMMENT ON FUNCTION app.is_system_admin() IS
    'Returns whether the current transaction has authenticated system-admin scope';
COMMENT ON FUNCTION app.can_access_carrier(UUID, UUID) IS
    'Checks an authenticated global user membership without exposing membership rows';
