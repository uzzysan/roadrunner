-- Transport schema: stops, routes, vehicles, GPS positions

-- Przystanki (z PostGIS)
CREATE TABLE stops (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    description TEXT,
    location GEOGRAPHY(POINT, 4326) NOT NULL,
    geofence_radius_m INT DEFAULT 50,
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Linie/trasy
CREATE TABLE routes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(100) NOT NULL,
    number VARCHAR(20) NOT NULL,
    description TEXT,
    type VARCHAR(20) NOT NULL DEFAULT 'regular', -- regular, school, night
    color VARCHAR(7) DEFAULT '#F7941D',
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Przystanki na trasie (kolejność)
CREATE TABLE route_stops (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    route_id UUID NOT NULL REFERENCES routes(id) ON DELETE CASCADE,
    stop_id UUID NOT NULL REFERENCES stops(id) ON DELETE CASCADE,
    sequence INT NOT NULL,
    scheduled_duration_from_start INT, -- sekundy od startu
    is_active BOOLEAN NOT NULL DEFAULT true,
    UNIQUE(route_id, sequence),
    UNIQUE(route_id, stop_id)
);

-- Pojazdy
CREATE TABLE vehicles (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    registration_number VARCHAR(20) UNIQUE NOT NULL,
    name VARCHAR(100),
    type VARCHAR(20) DEFAULT 'bus', -- bus, minibus, tram
    capacity INT,
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Przypisanie pojazdu do trasy (w danym momencie)
CREATE TABLE vehicle_assignments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    vehicle_id UUID NOT NULL REFERENCES vehicles(id),
    route_id UUID NOT NULL REFERENCES routes(id),
    driver_id UUID REFERENCES users(id),
    start_time TIMESTAMP WITH TIME ZONE NOT NULL,
    end_time TIMESTAMP WITH TIME ZONE,
    direction VARCHAR(10) NOT NULL DEFAULT 'forward', -- forward, backward
    is_active BOOLEAN NOT NULL DEFAULT true
);

-- Pozycje GPS (historia)
CREATE TABLE gps_positions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    vehicle_id UUID NOT NULL REFERENCES vehicles(id),
    assignment_id UUID REFERENCES vehicle_assignments(id),
    position GEOGRAPHY(POINT, 4326) NOT NULL,
    speed_kmh DECIMAL(5,2),
    heading INT, -- kierunek w stopniach 0-360
    accuracy_m DECIMAL(6,2),
    recorded_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Indeksy przestrzenne
CREATE INDEX idx_stops_location ON stops USING GIST(location);
CREATE INDEX idx_gps_positions_location ON gps_positions USING GIST(position);
CREATE INDEX idx_gps_positions_vehicle_time ON gps_positions(vehicle_id, recorded_at DESC);

-- Indeksy
CREATE INDEX idx_route_stops_route ON route_stops(route_id);
CREATE INDEX idx_route_stops_stop ON route_stops(stop_id);
CREATE INDEX idx_vehicle_assignments_vehicle ON vehicle_assignments(vehicle_id);
CREATE INDEX idx_vehicle_assignments_route ON vehicle_assignments(route_id);
CREATE INDEX idx_vehicle_assignments_active ON vehicle_assignments(is_active) WHERE is_active = true;
