-- Migration: Trip event log (Phase 2 — actual vs planned timetable)
-- Records actual stop arrivals/departures keyed to vehicle/route/date.
-- Designed for privacy minimisation: trip_events CASCADE-delete with the parent trip;
-- a maintenance job can DELETE FROM trips WHERE retain_until < CURRENT_DATE.

-- Trip lifecycle status
CREATE TYPE trip_status AS ENUM ('scheduled', 'in_progress', 'completed', 'cancelled', 'incomplete');

-- Stop event types (written by the Phase 2 geofence primitive)
CREATE TYPE trip_event_type AS ENUM ('arrived', 'departed');

-- One row per vehicle/route/date execution
CREATE TABLE trips (
    id              UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    vehicle_id      UUID        NOT NULL REFERENCES vehicles(id),
    route_id        UUID        NOT NULL REFERENCES routes(id),
    driver_id       UUID        REFERENCES drivers(id),          -- snapshot at trip start
    trip_date       DATE        NOT NULL,
    started_at      TIMESTAMPTZ,
    ended_at        TIMESTAMPTZ,
    status          trip_status NOT NULL DEFAULT 'scheduled',
    -- Privacy: delete this trip (+ its events via CASCADE) after retain_until.
    -- Default: trip_date + 30 days. Caller may override.
    retain_until    DATE        NOT NULL,
    created_at      TIMESTAMPTZ DEFAULT NOW(),
    updated_at      TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_trips_vehicle      ON trips(vehicle_id);
CREATE INDEX idx_trips_route        ON trips(route_id);
CREATE INDEX idx_trips_trip_date    ON trips(trip_date);
CREATE INDEX idx_trips_status       ON trips(status);
CREATE INDEX idx_trips_retain_until ON trips(retain_until);   -- for the cleanup job

-- Actual stop arrivals/departures; cascade-deleted with parent trip
CREATE TABLE trip_events (
    id              UUID              PRIMARY KEY DEFAULT gen_random_uuid(),
    trip_id         UUID              NOT NULL REFERENCES trips(id) ON DELETE CASCADE,
    stop_id         UUID              NOT NULL REFERENCES stops(id),
    schedule_id     UUID              REFERENCES schedules(id),  -- planned entry (nullable)
    event_type      trip_event_type   NOT NULL,
    occurred_at     TIMESTAMPTZ       NOT NULL,
    -- GPS position at event time (populated by geofence primitive; nullable until Phase 2)
    latitude        DOUBLE PRECISION  CHECK (latitude  >= -90  AND latitude  <= 90),
    longitude       DOUBLE PRECISION  CHECK (longitude >= -180 AND longitude <= 180),
    -- Signed seconds vs planned schedule: positive = late, negative = early; NULL if no schedule_id
    delay_seconds   INTEGER,
    created_at      TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_trip_events_trip        ON trip_events(trip_id);
CREATE INDEX idx_trip_events_stop        ON trip_events(stop_id);
CREATE INDEX idx_trip_events_occurred_at ON trip_events(occurred_at DESC);

-- updated_at trigger for trips
CREATE OR REPLACE FUNCTION update_trips_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_trips_updated_at
    BEFORE UPDATE ON trips
    FOR EACH ROW EXECUTE FUNCTION update_trips_updated_at();

COMMENT ON TABLE  trips                     IS 'Actual trip executions (one per vehicle/route/date run)';
COMMENT ON TABLE  trip_events               IS 'Actual stop arrivals/departures written by the geofence primitive';
COMMENT ON COLUMN trips.retain_until        IS 'Privacy boundary: DELETE trip + events after this date';
COMMENT ON COLUMN trip_events.delay_seconds IS 'Signed seconds vs planned schedule: positive=late, negative=early';