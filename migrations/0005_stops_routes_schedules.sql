-- Migration: Create stops, routes, and schedules tables with PostGIS
-- Created: 2026-03-26

-- Enable PostGIS extension (if not already enabled)
CREATE EXTENSION IF NOT EXISTS postgis;

DROP TABLE IF EXISTS route_stops CASCADE;
DROP TABLE IF EXISTS routes CASCADE;
DROP TABLE IF EXISTS stops CASCADE;

-- Create day_type enum for schedules
CREATE TYPE day_type AS ENUM ('weekday', 'saturday', 'sunday', 'holiday', 'everyday');

-- Create stops table with PostGIS Point
CREATE TABLE stops (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    description TEXT,
    location GEOGRAPHY(POINT, 4326) NOT NULL, -- PostGIS Point with SRID 4326 (WGS84)
    address VARCHAR(500),
    amenities TEXT[], -- Array of amenities: ['shelter', 'bench', 'timetable', 'wheelchair']
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Create spatial index for fast distance queries
CREATE INDEX idx_stops_location ON stops USING GIST(location);
CREATE INDEX idx_stops_active ON stops(is_active) WHERE is_active = true;
CREATE INDEX idx_stops_name ON stops(name);

-- Create routes table
CREATE TABLE routes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(100) NOT NULL,
    number VARCHAR(20) NOT NULL UNIQUE,
    description TEXT NOT NULL,
    color VARCHAR(7) DEFAULT '#2563EB',
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_routes_number ON routes(number);
CREATE INDEX idx_routes_active ON routes(is_active) WHERE is_active = true;

-- Create route_stops table (junction table with ordering)
CREATE TABLE route_stops (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    route_id UUID NOT NULL REFERENCES routes(id) ON DELETE CASCADE,
    stop_id UUID NOT NULL REFERENCES stops(id) ON DELETE CASCADE,
    stop_order INTEGER NOT NULL,
    sequence INTEGER, -- Backwards compatibility for 0002
    scheduled_duration_from_start INT,
    is_optional BOOLEAN DEFAULT false,
    is_active BOOLEAN NOT NULL DEFAULT true,
    UNIQUE(route_id, stop_order),
    UNIQUE(route_id, stop_id)
);

CREATE INDEX idx_route_stops_route ON route_stops(route_id);
CREATE INDEX idx_route_stops_stop ON route_stops(stop_id);

-- Create schedules table
CREATE TABLE schedules (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    route_id UUID NOT NULL REFERENCES routes(id) ON DELETE CASCADE,
    stop_id UUID NOT NULL REFERENCES stops(id) ON DELETE CASCADE,
    arrival_time TIME NOT NULL,
    departure_time TIME NOT NULL,
    day_type day_type NOT NULL,
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_schedules_route ON schedules(route_id);
CREATE INDEX idx_schedules_stop ON schedules(stop_id);
CREATE INDEX idx_schedules_day_type ON schedules(day_type);
CREATE INDEX idx_schedules_active ON schedules(is_active) WHERE is_active = true;
CREATE INDEX idx_schedules_arrival ON schedules(arrival_time);

-- Add comments for documentation
COMMENT ON TABLE stops IS 'Bus/tram stops with geographic location';
COMMENT ON TABLE routes IS 'Bus/tram routes';
COMMENT ON TABLE route_stops IS 'Mapping of stops to routes with ordering';
COMMENT ON TABLE schedules IS 'Timetable for routes at stops';
COMMENT ON COLUMN stops.location IS 'PostGIS Point (longitude, latitude) with SRID 4326';
COMMENT ON COLUMN stops.amenities IS 'Array of amenities: shelter, bench, timetable, wheelchair';

-- Sample data (optional - for testing)
-- Uncomment to add sample stops in Warsaw
/*
INSERT INTO stops (name, description, location, address, amenities) VALUES
('Centrum', 'Główny przystanek w centrum', 'SRID=4326;POINT(21.0118 52.2297)', 'ul. Marszałkowska 1', ARRAY['shelter', 'bench', 'timetable']),
('Metro Politechnika', 'Przy stacji metra', 'SRID=4326;POINT(21.0156 52.2204)', 'pl. Politechniki 1', ARRAY['shelter', 'timetable', 'wheelchair']),
('Lotnisko Chopina', 'Przystanek przy lotnisku', 'SRID=4326;POINT(20.9675 52.1672)', 'ul. Żwirki i Wigury 1', ARRAY['shelter', 'bench', 'timetable', 'wheelchair']);

INSERT INTO routes (name, number, description, color) VALUES
('Linia 175', '175', 'Centrum - Lotnisko Chopina', '#2563EB'),
('Linia 504', '504', 'Metro Politechnika - Centrum', '#EF4444');

INSERT INTO route_stops (route_id, stop_id, stop_order)
SELECT r.id, s.id, 1
FROM routes r, stops s
WHERE r.number = '175' AND s.name = 'Centrum';

INSERT INTO schedules (route_id, stop_id, arrival_time, departure_time, day_type)
SELECT r.id, s.id, '08:00'::TIME, '08:02'::TIME, 'weekday'
FROM routes r, stops s
WHERE r.number = '175' AND s.name = 'Centrum';
*/
