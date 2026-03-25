-- Migration: Faza 4 - System Awarii i Incydentów
-- Created: 2024-01-15

-- ============================================
-- TYPY WYLIECZEŃ (ENUMS)
-- ============================================

-- Typ incydentu
CREATE TYPE incident_type AS ENUM ('breakdown', 'accident', 'delay', 'route_change', 'other');

-- Poziom ważności
CREATE TYPE incident_severity AS ENUM ('low', 'medium', 'high', 'critical');

-- Status incydentu
CREATE TYPE incident_status AS ENUM ('reported', 'in_progress', 'resolved', 'cancelled');

-- ============================================
-- TABELA INCYDENTÓW (INCIDENTS)
-- ============================================

CREATE TABLE incidents (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    
    -- Powiązania
    vehicle_id UUID NOT NULL REFERENCES vehicles(id) ON DELETE CASCADE,
    driver_id UUID NOT NULL REFERENCES drivers(id),
    
    -- Klasyfikacja
    incident_type incident_type NOT NULL,
    severity incident_severity NOT NULL DEFAULT 'medium',
    
    -- Opis
    title VARCHAR(255) NOT NULL,
    description TEXT,
    
    -- Lokalizacja zdarzenia
    location GEOGRAPHY(POINT, 4326),
    
    -- Czasy
    reported_at TIMESTAMPTZ DEFAULT NOW(),
    resolved_at TIMESTAMPTZ,
    resolved_by UUID REFERENCES users(id),
    resolution_notes TEXT,
    estimated_resolution TIMESTAMPTZ,
    
    -- Status i zastępstwo
    status incident_status NOT NULL DEFAULT 'reported',
    replacement_vehicle_id UUID REFERENCES vehicles(id),
    
    -- Metadane
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Indeksy dla incidents
CREATE INDEX idx_incidents_vehicle ON incidents(vehicle_id);
CREATE INDEX idx_incidents_driver ON incidents(driver_id);
CREATE INDEX idx_incidents_status ON incidents(status);
CREATE INDEX idx_incidents_type ON incidents(incident_type);
CREATE INDEX idx_incidents_severity ON incidents(severity);
CREATE INDEX idx_incidents_location ON incidents USING GIST(location);
CREATE INDEX idx_incidents_reported ON incidents(reported_at);
CREATE INDEX idx_incidents_active ON incidents(status) WHERE status IN ('reported', 'in_progress');

-- Trigger do aktualizacji updated_at
CREATE TRIGGER trigger_incidents_updated_at
    BEFORE UPDATE ON incidents
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- ============================================
-- TABELA POWIADOMIEŃ O INCYDENTACH
-- ============================================

CREATE TABLE incident_notifications (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    incident_id UUID NOT NULL REFERENCES incidents(id) ON DELETE CASCADE,
    route_id UUID NOT NULL REFERENCES routes(id),
    
    -- Wiadomości wielojęzyczne
    message_pl TEXT NOT NULL,
    message_en TEXT NOT NULL,
    
    -- Metadane wysyłki
    sent_at TIMESTAMPTZ DEFAULT NOW(),
    affected_users_count INTEGER DEFAULT 0,
    
    -- Dodatkowe dane (np. szacowany czas opóźnienia)
    extra_data JSONB
);

-- Indeksy dla incident_notifications
CREATE INDEX idx_incident_notifications_incident ON incident_notifications(incident_id);
CREATE INDEX idx_incident_notifications_route ON incident_notifications(route_id);
CREATE INDEX idx_incident_notifications_sent ON incident_notifications(sent_at);

-- ============================================
-- TABELA WPŁYWU NA TRASY (INCIDENT_ROUTES)
-- ============================================
-- Śledzi które trasy są dotknięte przez incydent

CREATE TABLE incident_routes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    incident_id UUID NOT NULL REFERENCES incidents(id) ON DELETE CASCADE,
    route_id UUID NOT NULL REFERENCES routes(id),
    estimated_delay_minutes INTEGER CHECK (estimated_delay_minutes >= 0),
    notification_sent BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    
    UNIQUE(incident_id, route_id)
);

CREATE INDEX idx_incident_routes_incident ON incident_routes(incident_id);
CREATE INDEX idx_incident_routes_route ON incident_routes(route_id);

-- ============================================
-- FUNKCJE POMOCNICZE
-- ============================================

-- Funkcja do tworzenia incydentu i automatycznego wysłania powiadomień
CREATE OR REPLACE FUNCTION create_incident(
    p_vehicle_id UUID,
    p_driver_id UUID,
    p_incident_type incident_type,
    p_severity incident_severity,
    p_title VARCHAR(255),
    p_description TEXT DEFAULT NULL,
    p_latitude DOUBLE PRECISION DEFAULT NULL,
    p_longitude DOUBLE PRECISION DEFAULT NULL,
    p_estimated_resolution_minutes INTEGER DEFAULT NULL
)
RETURNS UUID AS $$
DECLARE
    v_incident_id UUID;
    v_route_id UUID;
    v_route_name VARCHAR(255);
    v_route_number VARCHAR(50);
    v_location GEOGRAPHY(POINT, 4326) := NULL;
    v_estimated_resolution TIMESTAMPTZ := NULL;
BEGIN
    -- Przygotuj lokalizację jeśli podano
    IF p_latitude IS NOT NULL AND p_longitude IS NOT NULL THEN
        v_location := ST_SetSRID(ST_MakePoint(p_longitude, p_latitude), 4326)::GEOGRAPHY;
    END IF;
    
    -- Oblicz szacowany czas rozwiązania
    IF p_estimated_resolution_minutes IS NOT NULL THEN
        v_estimated_resolution := NOW() + (p_estimated_resolution_minutes || ' minutes')::INTERVAL;
    END IF;
    
    -- Utwórz incydent
    INSERT INTO incidents (
        vehicle_id, driver_id, incident_type, severity,
        title, description, location, estimated_resolution
    ) VALUES (
        p_vehicle_id, p_driver_id, p_incident_type, p_severity,
        p_title, p_description, v_location, v_estimated_resolution
    ) RETURNING id INTO v_incident_id;
    
    -- Pobierz dane trasy pojazdu
    SELECT r.id, r.name, r.number
    INTO v_route_id, v_route_name, v_route_number
    FROM vehicles v
    JOIN routes r ON v.current_route_id = r.id
    WHERE v.id = p_vehicle_id;
    
    -- Jeśli pojazd ma przypisaną trasę, utwórz powiadomienie
    IF v_route_id IS NOT NULL THEN
        -- Dodaj wpis do incident_routes
        INSERT INTO incident_routes (incident_id, route_id, estimated_delay_minutes)
        VALUES (v_incident_id, v_route_id, p_estimated_resolution_minutes);
        
        -- Wygeneruj i zapisz powiadomienie
        -- (w rzeczywistości tutaj byłaby też wysyłka push/SMS)
        PERFORM create_incident_notification(
            v_incident_id, 
            v_route_id, 
            v_route_name, 
            v_route_number,
            p_incident_type,
            p_severity,
            p_estimated_resolution_minutes
        );
    END IF;
    
    -- Jeśli awaria krytyczna, oznacz pojazd jako uszkodzony
    IF p_severity = 'critical' AND p_incident_type IN ('breakdown', 'accident') THEN
        UPDATE vehicles 
        SET status = 'broken',
            updated_at = NOW()
        WHERE id = p_vehicle_id;
    END IF;
    
    RETURN v_incident_id;
END;
$$ LANGUAGE plpgsql;

-- Funkcja do generowania powiadomienia o incydencie
CREATE OR REPLACE FUNCTION create_incident_notification(
    p_incident_id UUID,
    p_route_id UUID,
    p_route_name VARCHAR(255),
    p_route_number VARCHAR(50),
    p_incident_type incident_type,
    p_severity incident_severity,
    p_delay_minutes INTEGER DEFAULT NULL
)
RETURNS UUID AS $$
DECLARE
    v_notification_id UUID;
    v_message_pl TEXT;
    v_message_en TEXT;
BEGIN
    -- Wygeneruj wiadomości w zależności od typu
    CASE p_incident_type
        WHEN 'breakdown' THEN
            IF p_delay_minutes IS NOT NULL THEN
                v_message_pl := format('Awaria pojazdu na linii %s (%s). Opóźnienie ok. %s min.', 
                    p_route_number, p_route_name, p_delay_minutes);
                v_message_en := format('Vehicle breakdown on line %s (%s). Delay approx. %s min.', 
                    p_route_number, p_route_name, p_delay_minutes);
            ELSE
                v_message_pl := format('Awaria pojazdu na linii %s (%s). Trwają prace naprawcze.', 
                    p_route_number, p_route_name);
                v_message_en := format('Vehicle breakdown on line %s (%s). Repair work in progress.', 
                    p_route_number, p_route_name);
            END IF;
            
        WHEN 'delay' THEN
            IF p_delay_minutes IS NOT NULL THEN
                v_message_pl := format('Opóźnienie na linii %s (%s). Opóźnienie ok. %s min.', 
                    p_route_number, p_route_name, p_delay_minutes);
                v_message_en := format('Delay on line %s (%s). Delay approx. %s min.', 
                    p_route_number, p_route_name, p_delay_minutes);
            ELSE
                v_message_pl := format('Opóźnienie na linii %s (%s).', p_route_number, p_route_name);
                v_message_en := format('Delay on line %s (%s).', p_route_number, p_route_name);
            END IF;
            
        WHEN 'accident' THEN
            v_message_pl := format('Incydent na linii %s (%s). Pojazd zastępczy został wysłany.', 
                p_route_number, p_route_name);
            v_message_en := format('Incident on line %s (%s). Replacement vehicle has been dispatched.', 
                p_route_number, p_route_name);
            
        WHEN 'route_change' THEN
            v_message_pl := format('Zmiana trasy linii %s (%s). Sprawdź szczegóły w aplikacji.', 
                p_route_number, p_route_name);
            v_message_en := format('Route change for line %s (%s). Check details in the app.', 
                p_route_number, p_route_name);
            
        ELSE
            v_message_pl := format('Zdarzenie na linii %s (%s). Możliwe opóźnienia.', 
                p_route_number, p_route_name);
            v_message_en := format('Incident on line %s (%s). Possible delays.', 
                p_route_number, p_route_name);
    END CASE;
    
    -- Zapisz powiadomienie
    INSERT INTO incident_notifications (
        incident_id, route_id, message_pl, message_en,
        extra_data
    ) VALUES (
        p_incident_id, p_route_id, v_message_pl, v_message_en,
        jsonb_build_object(
            'severity', p_severity,
            'delay_minutes', p_delay_minutes,
            'incident_type', p_incident_type
        )
    ) RETURNING id INTO v_notification_id;
    
    -- Oznacz że powiadomienie zostało wysłane
    UPDATE incident_routes 
    SET notification_sent = TRUE
    WHERE incident_id = p_incident_id AND route_id = p_route_id;
    
    RETURN v_notification_id;
END;
$$ LANGUAGE plpgsql;

-- Funkcja do rozwiązywania incydentu
CREATE OR REPLACE FUNCTION resolve_incident(
    p_incident_id UUID,
    p_resolved_by UUID,
    p_resolution_notes TEXT DEFAULT NULL
)
RETURNS BOOLEAN AS $$
DECLARE
    v_vehicle_id UUID;
    v_replacement_vehicle_id UUID;
BEGIN
    -- Pobierz dane incydentu
    SELECT vehicle_id, replacement_vehicle_id 
    INTO v_vehicle_id, v_replacement_vehicle_id
    FROM incidents WHERE id = p_incident_id;
    
    -- Zaktualizuj incydent
    UPDATE incidents
    SET status = 'resolved',
        resolved_at = NOW(),
        resolved_by = p_resolved_by,
        resolution_notes = p_resolution_notes,
        updated_at = NOW()
    WHERE id = p_incident_id;
    
    -- Przywróć pojazd do aktywnych jeśli był oznaczony jako uszkodzony
    UPDATE vehicles
    SET status = 'active',
        updated_at = NOW()
    WHERE id = v_vehicle_id AND status = 'broken';
    
    -- Jeśli był pojazd zastępczy, zwolnij go
    IF v_replacement_vehicle_id IS NOT NULL THEN
        UPDATE vehicles
        SET current_route_id = NULL,
            current_driver_id = NULL,
            updated_at = NOW()
        WHERE id = v_replacement_vehicle_id;
    END IF;
    
    RETURN TRUE;
END;
$$ LANGUAGE plpgsql;

-- Funkcja do przypisywania pojazdu zastępczego
CREATE OR REPLACE FUNCTION assign_replacement_vehicle(
    p_incident_id UUID,
    p_replacement_vehicle_id UUID
)
RETURNS BOOLEAN AS $$
DECLARE
    v_vehicle_id UUID;
    v_route_id UUID;
    v_driver_id UUID;
BEGIN
    -- Pobierz dane incydentu
    SELECT vehicle_id INTO v_vehicle_id
    FROM incidents WHERE id = p_incident_id;
    
    -- Pobierz trasę i kierowcę z uszkodzonego pojazdu
    SELECT current_route_id, current_driver_id
    INTO v_route_id, v_driver_id
    FROM vehicles WHERE id = v_vehicle_id;
    
    -- Sprawdź czy pojazd zastępczy jest dostępny
    IF NOT EXISTS (
        SELECT 1 FROM vehicles 
        WHERE id = p_replacement_vehicle_id 
          AND status = 'active'
          AND current_route_id IS NULL
    ) THEN
        RAISE EXCEPTION 'Replacement vehicle is not available';
    END IF;
    
    -- Przypisz pojazd zastępczy do trasy i kierowcy
    UPDATE vehicles
    SET current_route_id = v_route_id,
        current_driver_id = v_driver_id,
        updated_at = NOW()
    WHERE id = p_replacement_vehicle_id;
    
    -- Zapisz informację o zastępstwie w incydencie
    UPDATE incidents
    SET replacement_vehicle_id = p_replacement_vehicle_id,
        status = 'in_progress',
        updated_at = NOW()
    WHERE id = p_incident_id;
    
    -- Odłącz kierowcę od uszkodzonego pojazdu
    UPDATE vehicles
    SET current_driver_id = NULL,
        updated_at = NOW()
    WHERE id = v_vehicle_id;
    
    RETURN TRUE;
END;
$$ LANGUAGE plpgsql;

-- ============================================
-- WIDOKI (VIEWS)
-- ============================================

-- Widok aktywnych incydentów z danymi pojazdu i trasy
CREATE VIEW active_incidents_view AS
SELECT 
    i.*,
    v.registration_number,
    v.brand as vehicle_brand,
    v.model as vehicle_model,
    v.current_route_id,
    r.name as route_name,
    r.number as route_number,
    r.color as route_color,
    CONCAT(u.first_name, ' ', u.last_name) as driver_name,
    u.phone as driver_phone,
    CASE 
        WHEN i.resolved_at IS NOT NULL THEN 
            EXTRACT(EPOCH FROM (i.resolved_at - i.reported_at)) / 60
        ELSE 
            EXTRACT(EPOCH FROM (NOW() - i.reported_at)) / 60
    END as duration_minutes
FROM incidents i
JOIN vehicles v ON i.vehicle_id = v.id
JOIN drivers d ON i.driver_id = d.id
JOIN users u ON d.user_id = u.id
LEFT JOIN routes r ON v.current_route_id = r.id
WHERE i.status IN ('reported', 'in_progress');

-- Widok statystyk incydentów
CREATE VIEW incident_stats_view AS
SELECT 
    COUNT(*) as total_incidents,
    COUNT(*) FILTER (WHERE status IN ('reported', 'in_progress')) as active_incidents,
    COUNT(*) FILTER (WHERE status = 'resolved' AND resolved_at >= CURRENT_DATE) as resolved_today,
    COUNT(*) FILTER (WHERE severity = 'low') as low_count,
    COUNT(*) FILTER (WHERE severity = 'medium') as medium_count,
    COUNT(*) FILTER (WHERE severity = 'high') as high_count,
    COUNT(*) FILTER (WHERE severity = 'critical') as critical_count,
    AVG(
        CASE 
            WHEN resolved_at IS NOT NULL THEN 
                EXTRACT(EPOCH FROM (resolved_at - reported_at)) / 60
            ELSE NULL
        END
    ) as avg_resolution_minutes
FROM incidents;

-- Widok powiadomień z danymi incydentu
CREATE VIEW incident_notifications_view AS
SELECT 
    n.*,
    i.incident_type,
    i.severity,
    i.status as incident_status,
    v.registration_number,
    r.name as route_name,
    r.number as route_number
FROM incident_notifications n
JOIN incidents i ON n.incident_id = i.id
JOIN vehicles v ON i.vehicle_id = v.id
JOIN routes r ON n.route_id = r.id;

-- ============================================
-- KOMENTARZE
-- ============================================

COMMENT ON TABLE incidents IS 'Zgłoszone awarie, opóźnienia i inne incydenty';
COMMENT ON TABLE incident_notifications IS 'Powiadomienia wysłane do pasażerów o incydentach';
COMMENT ON TABLE incident_routes IS 'Trasy dotknięte przez incydenty';

COMMENT ON COLUMN incidents.severity IS 'Poziom ważności: low/medium/high/critical';
COMMENT ON COLUMN incidents.replacement_vehicle_id IS 'ID pojazdu zastępczego jeśli przypisany';
COMMENT ON COLUMN incidents.estimated_resolution IS 'Szacowany czas rozwiązania problemu';
