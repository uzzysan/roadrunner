-- Migration: Faza 4 - Zarządzanie Flotą (Vehicles & Drivers)
-- Created: 2024-01-15

-- ============================================
-- TYPY WYLIECZEŃ (ENUMS)
-- ============================================

-- Typ pojazdu
CREATE TYPE vehicle_type AS ENUM ('bus', 'minibus', 'coach', 'tram', 'trolleybus');

-- Typ paliwa
CREATE TYPE fuel_type AS ENUM ('diesel', 'electric', 'hybrid', 'cng', 'hydrogen');

-- Status pojazdu
CREATE TYPE vehicle_status AS ENUM ('active', 'maintenance', 'retired', 'broken');

-- Status kierowcy
CREATE TYPE driver_status AS ENUM ('active', 'on_leave', 'suspended', 'inactive');

-- ============================================
-- TABELA POJAZDÓW (VEHICLES)
-- ============================================

DROP TABLE IF EXISTS vehicle_assignments CASCADE;
DROP TABLE IF EXISTS gps_positions CASCADE;
DROP TABLE IF EXISTS vehicles CASCADE;

CREATE TABLE vehicles (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    
    -- Dane pojazdu
    registration_number VARCHAR(20) UNIQUE NOT NULL,
    vin VARCHAR(17) UNIQUE,
    brand VARCHAR(100) NOT NULL,
    model VARCHAR(100) NOT NULL,
    year INTEGER CHECK (year >= 1900 AND year <= 2100),
    capacity INTEGER NOT NULL DEFAULT 50 CHECK (capacity > 0),
    
    -- Klasyfikacja
    vehicle_type vehicle_type NOT NULL DEFAULT 'bus',
    fuel_type fuel_type NOT NULL DEFAULT 'diesel',
    status vehicle_status NOT NULL DEFAULT 'active',
    
    -- GPS tracking
    gps_device_id VARCHAR(100),
    last_location GEOGRAPHY(POINT, 4326),
    last_location_at TIMESTAMPTZ,
    
    -- Aktualne przypisanie
    current_driver_id UUID REFERENCES users(id) ON DELETE SET NULL,
    current_route_id UUID REFERENCES routes(id) ON DELETE SET NULL,
    
    -- Metadane
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    
    -- Constraints
    CONSTRAINT valid_registration CHECK (registration_number ~ '^[A-Z0-9\- ]+$')
);

-- Indeksy dla vehicles
CREATE INDEX idx_vehicles_status ON vehicles(status);
CREATE INDEX idx_vehicles_type ON vehicles(vehicle_type);
CREATE INDEX idx_vehicles_location ON vehicles USING GIST(last_location);
CREATE INDEX idx_vehicles_driver ON vehicles(current_driver_id);
CREATE INDEX idx_vehicles_route ON vehicles(current_route_id);
CREATE INDEX idx_vehicles_gps ON vehicles(gps_device_id) WHERE gps_device_id IS NOT NULL;

-- Trigger do aktualizacji updated_at
CREATE OR REPLACE FUNCTION update_vehicles_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_vehicles_updated_at
    BEFORE UPDATE ON vehicles
    FOR EACH ROW
    EXECUTE FUNCTION update_vehicles_updated_at();

-- ============================================
-- TABELA KIEROWCÓW (DRIVERS)
-- ============================================

CREATE TABLE drivers (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    
    -- Powiązanie z użytkownikiem
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    
    -- Dane pracownicze
    employee_id VARCHAR(50) UNIQUE,
    
    -- Prawo jazdy
    license_number VARCHAR(50) NOT NULL,
    license_categories TEXT[] NOT NULL DEFAULT '{}',
    license_expiry DATE NOT NULL,
    
    -- Kontakt
    phone VARCHAR(20) NOT NULL,
    emergency_contact VARCHAR(100),
    
    -- Status i przypisanie
    status driver_status NOT NULL DEFAULT 'active',
    assigned_vehicle_id UUID REFERENCES vehicles(id) ON DELETE SET NULL,
    
    -- Metadane
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    
    -- Constraints
    UNIQUE(user_id),
    CONSTRAINT valid_phone CHECK (phone ~ '^[+0-9\-\s]+$')
);

-- Indeksy dla drivers
CREATE INDEX idx_drivers_user ON drivers(user_id);
CREATE INDEX idx_drivers_status ON drivers(status);
CREATE INDEX idx_drivers_vehicle ON drivers(assigned_vehicle_id);
CREATE INDEX idx_drivers_license_expiry ON drivers(license_expiry);

-- Trigger do aktualizacji updated_at
CREATE TRIGGER trigger_drivers_updated_at
    BEFORE UPDATE ON drivers
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- ============================================
-- TABELA HISTORII LOKALIZACJI (VEHICLE_LOCATIONS)
-- ============================================
-- Opcjonalna tabela do przechowywania historii lokalizacji
-- Można użyć do analizy tras i raportowania

CREATE TABLE vehicle_locations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    vehicle_id UUID NOT NULL REFERENCES vehicles(id) ON DELETE CASCADE,
    location GEOGRAPHY(POINT, 4326) NOT NULL,
    speed DOUBLE PRECISION CHECK (speed >= 0 AND speed <= 300),
    heading DOUBLE PRECISION CHECK (heading >= 0 AND heading < 360),
    next_stop_id UUID REFERENCES stops(id),
    eta_seconds INTEGER CHECK (eta_seconds >= 0),
    recorded_at TIMESTAMPTZ DEFAULT NOW(),
    
    -- Metadane urządzenia
    gps_accuracy DOUBLE PRECISION,
    battery_level INTEGER CHECK (battery_level >= 0 AND battery_level <= 100)
);

-- Indeksy dla vehicle_locations
CREATE INDEX idx_vehicle_locations_vehicle ON vehicle_locations(vehicle_id);
CREATE INDEX idx_vehicle_locations_recorded ON vehicle_locations(recorded_at);
CREATE INDEX idx_vehicle_locations_location ON vehicle_locations USING GIST(location);

-- Partycjonowanie dla dużej ilości danych (opcjonalnie)
-- CREATE TABLE vehicle_locations PARTITION BY RANGE (recorded_at);

-- ============================================
-- FUNKCJE POMOCNICZE
-- ============================================

-- Funkcja do pobierania dostępnych pojazdów (nieprzypisanych lub wolnych)
CREATE OR REPLACE FUNCTION get_available_vehicles()
RETURNS TABLE (
    id UUID,
    registration_number VARCHAR(20),
    brand VARCHAR(100),
    model VARCHAR(100),
    capacity INTEGER,
    vehicle_type vehicle_type
) AS $$
BEGIN
    RETURN QUERY
    SELECT 
        v.id,
        v.registration_number,
        v.brand,
        v.model,
        v.capacity,
        v.vehicle_type
    FROM vehicles v
    WHERE v.status = 'active'
      AND (v.current_driver_id IS NULL OR v.current_route_id IS NULL);
END;
$$ LANGUAGE plpgsql;

-- Funkcja do sprawdzania ważności prawa jazdy kierowcy
CREATE OR REPLACE FUNCTION is_driver_license_valid(driver_uuid UUID)
RETURNS BOOLEAN AS $$
DECLARE
    expiry_date DATE;
BEGIN
    SELECT license_expiry INTO expiry_date
    FROM drivers
    WHERE id = driver_uuid;
    
    IF expiry_date IS NULL THEN
        RETURN FALSE;
    END IF;
    
    RETURN expiry_date >= CURRENT_DATE;
END;
$$ LANGUAGE plpgsql;

-- Funkcja do przypisania kierowcy do pojazdu
CREATE OR REPLACE FUNCTION assign_driver_to_vehicle(
    p_driver_id UUID,
    p_vehicle_id UUID
)
RETURNS BOOLEAN AS $$
DECLARE
    v_status vehicle_status;
    d_status driver_status;
BEGIN
    -- Sprawdź status pojazdu
    SELECT status INTO v_status
    FROM vehicles WHERE id = p_vehicle_id;
    
    IF v_status != 'active' THEN
        RAISE EXCEPTION 'Vehicle is not active';
    END IF;
    
    -- Sprawdź status kierowcy
    SELECT status INTO d_status
    FROM drivers WHERE id = p_driver_id;
    
    IF d_status != 'active' THEN
        RAISE EXCEPTION 'Driver is not active';
    END IF;
    
    -- Sprawdź ważność prawa jazdy
    IF NOT is_driver_license_valid(p_driver_id) THEN
        RAISE EXCEPTION 'Driver license is expired';
    END IF;
    
    -- Przypisz kierowcę do pojazdu
    UPDATE vehicles
    SET current_driver_id = p_driver_id,
        updated_at = NOW()
    WHERE id = p_vehicle_id;
    
    -- Zaktualizuj przypisanie u kierowcy
    UPDATE drivers
    SET assigned_vehicle_id = p_vehicle_id,
        updated_at = NOW()
    WHERE id = p_driver_id;
    
    RETURN TRUE;
END;
$$ LANGUAGE plpgsql;

-- Funkcja do usuwania przypisania kierowcy
CREATE OR REPLACE FUNCTION unassign_driver_from_vehicle(p_vehicle_id UUID)
RETURNS BOOLEAN AS $$
DECLARE
    v_driver_id UUID;
BEGIN
    -- Pobierz aktualnego kierowcę
    SELECT current_driver_id INTO v_driver_id
    FROM vehicles WHERE id = p_vehicle_id;
    
    -- Usuń przypisanie z pojazdu
    UPDATE vehicles
    SET current_driver_id = NULL,
        current_route_id = NULL,
        updated_at = NOW()
    WHERE id = p_vehicle_id;
    
    -- Usuń przypisanie u kierowcy
    IF v_driver_id IS NOT NULL THEN
        UPDATE drivers
        SET assigned_vehicle_id = NULL,
            updated_at = NOW()
        WHERE id = v_driver_id;
    END IF;
    
    RETURN TRUE;
END;
$$ LANGUAGE plpgsql;

-- ============================================
-- WIDOKI (VIEWS)
-- ============================================

-- Widok pojazdów z danymi kierowcy
CREATE VIEW vehicle_driver_view AS
SELECT 
    v.*,
    d.id as driver_id,
    d.license_number as driver_license,
    d.phone as driver_phone,
    d.status as driver_status,
    u.first_name as driver_first_name,
    u.last_name as driver_last_name,
    u.email as driver_email
FROM vehicles v
LEFT JOIN drivers d ON v.current_driver_id = d.id
LEFT JOIN users u ON d.user_id = u.id;

-- Widok kierowców z danymi pojazdu
CREATE VIEW driver_vehicle_view AS
SELECT 
    d.*,
    v.registration_number as vehicle_registration,
    v.brand as vehicle_brand,
    v.model as vehicle_model,
    v.status as vehicle_status,
    u.first_name,
    u.last_name,
    u.email,
    CONCAT(u.first_name, ' ', u.last_name) as full_name
FROM drivers d
LEFT JOIN vehicles v ON d.assigned_vehicle_id = v.id
JOIN users u ON d.user_id = u.id;

-- Widok aktywnych pojazdów na trasach
CREATE VIEW active_vehicles_view AS
SELECT 
    v.id,
    v.registration_number,
    v.brand,
    v.model,
    v.last_location,
    v.last_location_at,
    r.id as route_id,
    r.name as route_name,
    r.number as route_number,
    r.color as route_color,
    d.id as driver_id,
    CONCAT(u.first_name, ' ', u.last_name) as driver_name
FROM vehicles v
JOIN routes r ON v.current_route_id = r.id
LEFT JOIN drivers d ON v.current_driver_id = d.id
LEFT JOIN users u ON d.user_id = u.id
WHERE v.status = 'active'
  AND v.current_route_id IS NOT NULL;

-- ============================================
-- PRZYKŁADOWE DANE (OPCJONALNIE)
-- ============================================

-- Dodaj przykładowe pojazdy (po utworzeniu użytkowników)
-- INSERT INTO vehicles (registration_number, brand, model, year, capacity, vehicle_type, fuel_type, status)
-- VALUES 
--     ('WX 12345', 'Mercedes-Benz', 'Citaro', 2020, 85, 'bus', 'diesel', 'active'),
--     ('WX 67890', 'MAN', 'Lions City', 2021, 90, 'bus', 'electric', 'active'),
--     ('WX 11111', 'Solaris', 'Urbino 12', 2019, 80, 'bus', 'cng', 'active');

-- ============================================
-- KOMENTARZE
-- ============================================

COMMENT ON TABLE vehicles IS 'Flota pojazdów transportowych';
COMMENT ON TABLE drivers IS 'Kierowcy z uprawnieniami';
COMMENT ON TABLE vehicle_locations IS 'Historia lokalizacji GPS pojazdów';

COMMENT ON COLUMN vehicles.registration_number IS 'Numer rejestracyjny pojazdu';
COMMENT ON COLUMN vehicles.vin IS 'Numer VIN (Vehicle Identification Number)';
COMMENT ON COLUMN vehicles.gps_device_id IS 'Identyfikator urządzenia GPS';
COMMENT ON COLUMN vehicles.last_location IS 'Ostatnia znana lokalizacja (PostGIS POINT)';
COMMENT ON COLUMN drivers.license_categories IS 'Kategorie prawa jazdy (np. ["D", "DE"])';
COMMENT ON COLUMN drivers.license_expiry IS 'Data ważności prawa jazdy';
