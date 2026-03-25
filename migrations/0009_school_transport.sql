-- Migration: Faza 4 - School Transport (Dzieci, Obecność, Powiadomienia)
-- Created: 2024-01-15

-- ============================================
-- TYPY WYLIECZEŃ (ENUMS)
-- ============================================

-- Status rejestracji dziecka
CREATE TYPE child_status AS ENUM ('active', 'inactive', 'suspended');

-- Status obecności
CREATE TYPE attendance_status AS ENUM ('scheduled', 'picked_up', 'dropped_off', 'absent', 'cancelled');

-- Metoda potwierdzenia
CREATE TYPE confirmation_method AS ENUM ('qr_code', 'manual', 'auto_gps');

-- Typ powiadomienia dla rodzica
CREATE TYPE parent_notification_type AS ENUM (
    'child_picked_up',      -- Dziecko wsiadło
    'child_dropped_off',    -- Dziecko wysiadło
    'bus_delayed',          -- Autobus opóźniony
    'bus_approaching',      -- Autobus zbliża się
    'route_changed',        -- Zmiana trasy
    'incident',             -- Incydent
    'general'               -- Ogólne
);

-- Kanał powiadomienia
CREATE TYPE notification_channel AS ENUM ('push', 'sms', 'email');

-- ============================================
-- TABELA REJESTRACJI DZIECKA
-- ============================================

CREATE TABLE child_registrations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    
    -- Powiązanie z rodzicem
    parent_user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    
    -- Dane dziecka
    child_first_name VARCHAR(100) NOT NULL,
    child_last_name VARCHAR(100) NOT NULL,
    child_birth_date DATE NOT NULL,
    
    -- Szkoła
    school_name VARCHAR(255) NOT NULL,
    school_address TEXT,
    
    -- Przypisanie do trasy
    assigned_route_id UUID REFERENCES routes(id),
    pickup_stop_id UUID REFERENCES stops(id),
    dropoff_stop_id UUID REFERENCES stops(id),
    
    -- Kod QR (unikalny dla dziecka)
    qr_code VARCHAR(255) UNIQUE NOT NULL,
    qr_code_data TEXT NOT NULL,  -- Dane do wygenerowania QR
    
    -- Opcjonalne zdjęcie
    photo_url TEXT,
    
    -- Status i notatki
    status child_status NOT NULL DEFAULT 'active',
    notes TEXT,
    
    -- Metadane
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Indeksy dla child_registrations
CREATE INDEX idx_children_parent ON child_registrations(parent_user_id);
CREATE INDEX idx_children_route ON child_registrations(assigned_route_id);
CREATE INDEX idx_children_pickup ON child_registrations(pickup_stop_id);
CREATE INDEX idx_children_qr ON child_registrations(qr_code);
CREATE INDEX idx_children_status ON child_registrations(status);

-- Trigger do aktualizacji updated_at
CREATE TRIGGER trigger_children_updated_at
    BEFORE UPDATE ON child_registrations
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- ============================================
-- TABELA OBECNOŚCI DZIECKA
-- ============================================

CREATE TABLE child_attendance (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    
    -- Powiązania
    child_id UUID NOT NULL REFERENCES child_registrations(id) ON DELETE CASCADE,
    route_id UUID NOT NULL REFERENCES routes(id),
    vehicle_id UUID NOT NULL REFERENCES vehicles(id),
    driver_id UUID NOT NULL REFERENCES drivers(id),
    
    -- Wsiadanie
    pickup_stop_id UUID REFERENCES stops(id),
    pickup_time TIMESTAMPTZ,
    
    -- Wysiadanie
    dropoff_stop_id UUID REFERENCES stops(id),
    dropoff_time TIMESTAMPTZ,
    
    -- Status i potwierdzenie
    status attendance_status NOT NULL DEFAULT 'scheduled',
    confirmed_by confirmation_method,
    
    -- Powiadomienia
    parent_notified_pickup BOOLEAN DEFAULT FALSE,
    parent_notified_dropoff BOOLEAN DEFAULT FALSE,
    
    -- Data (dla łatwiejszego zapytania)
    date DATE NOT NULL DEFAULT CURRENT_DATE,
    
    -- Metadane
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Indeksy dla child_attendance
CREATE INDEX idx_attendance_child ON child_attendance(child_id);
CREATE INDEX idx_attendance_date ON child_attendance(date);
CREATE INDEX idx_attendance_route ON child_attendance(route_id);
CREATE INDEX idx_attendance_vehicle ON child_attendance(vehicle_id);
CREATE INDEX idx_attendance_driver ON child_attendance(driver_id);
CREATE INDEX idx_attendance_status ON child_attendance(status);
CREATE INDEX idx_attendance_today ON child_attendance(date, route_id, driver_id);

-- Unikalny indeks - jeden wpis na dzień i dziecko
CREATE UNIQUE INDEX idx_attendance_unique 
ON child_attendance(child_id, date) 
WHERE status != 'cancelled';

-- ============================================
-- TABELA POWIADOMIEŃ DLA RODZICÓW
-- ============================================

CREATE TABLE parent_notifications (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    
    -- Powiązania
    parent_user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    child_id UUID REFERENCES child_registrations(id) ON DELETE CASCADE,
    
    -- Treść
    notification_type parent_notification_type NOT NULL,
    title VARCHAR(255) NOT NULL,
    message TEXT NOT NULL,
    data JSONB,  -- Dodatkowe dane (np. lokalizacja, czas)
    
    -- Status
    sent_at TIMESTAMPTZ DEFAULT NOW(),
    read_at TIMESTAMPTZ,
    channel notification_channel NOT NULL DEFAULT 'push',
    
    -- Metadane urządzenia
    device_token VARCHAR(255)
);

-- Indeksy dla parent_notifications
CREATE INDEX idx_notifications_parent ON parent_notifications(parent_user_id);
CREATE INDEX idx_notifications_child ON parent_notifications(child_id);
CREATE INDEX idx_notifications_unread ON parent_notifications(parent_user_id, read_at) WHERE read_at IS NULL;
CREATE INDEX idx_notifications_sent ON parent_notifications(sent_at);
CREATE INDEX idx_notifications_type ON parent_notifications(notification_type);

-- ============================================
-- FUNKCJE POMOCNICZE
-- ============================================

-- Funkcja do generowania unikalnego kodu QR
CREATE OR REPLACE FUNCTION generate_child_qr_code()
RETURNS VARCHAR(255) AS $$
DECLARE
    v_code VARCHAR(255);
    v_exists BOOLEAN;
BEGIN
    LOOP
        -- Generuj losowy kod (prefix + UUID fragment)
        v_code := 'CHILD-' || upper(substring(md5(random()::text), 1, 12));
        
        -- Sprawdź czy kod już istnieje
        SELECT EXISTS(SELECT 1 FROM child_registrations WHERE qr_code = v_code) INTO v_exists;
        
        -- Jeśli nie istnieje, zwróć go
        IF NOT v_exists THEN
            RETURN v_code;
        END IF;
    END LOOP;
END;
$$ LANGUAGE plpgsql;

-- Funkcja do rejestracji dziecka
CREATE OR REPLACE FUNCTION register_child(
    p_parent_id UUID,
    p_first_name VARCHAR(100),
    p_last_name VARCHAR(100),
    p_birth_date DATE,
    p_school_name VARCHAR(255),
    p_school_address TEXT DEFAULT NULL,
    p_route_id UUID DEFAULT NULL,
    p_pickup_stop_id UUID DEFAULT NULL,
    p_dropoff_stop_id UUID DEFAULT NULL,
    p_notes TEXT DEFAULT NULL
)
RETURNS UUID AS $$
DECLARE
    v_child_id UUID;
    v_qr_code VARCHAR(255);
    v_qr_data TEXT;
BEGIN
    -- Wygeneruj kod QR
    v_qr_code := generate_child_qr_code();
    
    -- Przygotuj dane QR (JSON z kluczowymi informacjami)
    v_qr_data := jsonb_build_object(
        'child_id', v_child_id,
        'qr_code', v_qr_code,
        'name', p_first_name || ' ' || p_last_name,
        'school', p_school_name,
        'version', '1.0'
    )::TEXT;
    
    -- Utwórz rejestrację
    INSERT INTO child_registrations (
        parent_user_id, child_first_name, child_last_name, child_birth_date,
        school_name, school_address, assigned_route_id, 
        pickup_stop_id, dropoff_stop_id, qr_code, qr_code_data, notes
    ) VALUES (
        p_parent_id, p_first_name, p_last_name, p_birth_date,
        p_school_name, p_school_address, p_route_id,
        p_pickup_stop_id, p_dropoff_stop_id, v_qr_code, v_qr_data, p_notes
    ) RETURNING id INTO v_child_id;
    
    -- Zaktualizuj QR data z prawdziwym ID
    UPDATE child_registrations
    SET qr_code_data = jsonb_build_object(
        'child_id', v_child_id,
        'qr_code', v_qr_code,
        'name', p_first_name || ' ' || p_last_name,
        'school', p_school_name,
        'version', '1.0'
    )::TEXT
    WHERE id = v_child_id;
    
    RETURN v_child_id;
END;
$$ LANGUAGE plpgsql;

-- Funkcja do tworzenia dziennego rekordu obecności
CREATE OR REPLACE FUNCTION create_daily_attendance(
    p_child_id UUID,
    p_date DATE DEFAULT CURRENT_DATE
)
RETURNS UUID AS $$
DECLARE
    v_attendance_id UUID;
    v_route_id UUID;
    v_vehicle_id UUID;
    v_driver_id UUID;
BEGIN
    -- Pobierz przypisanie dziecka
    SELECT assigned_route_id INTO v_route_id
    FROM child_registrations
    WHERE id = p_child_id AND status = 'active';
    
    -- Jeśli nie ma trasy, nie twórz obecności
    IF v_route_id IS NULL THEN
        RETURN NULL;
    END IF;
    
    -- Pobierz pojazd i kierowcę przypisanego do trasy
    SELECT id, current_driver_id 
    INTO v_vehicle_id, v_driver_id
    FROM vehicles
    WHERE current_route_id = v_route_id
    LIMIT 1;
    
    -- Utwórz rekord obecności
    INSERT INTO child_attendance (
        child_id, route_id, vehicle_id, driver_id, date, status
    ) VALUES (
        p_child_id, v_route_id, v_vehicle_id, v_driver_id, p_date, 'scheduled'
    )
    ON CONFLICT (child_id, date) WHERE status != 'cancelled' DO NOTHING
    RETURNING id INTO v_attendance_id;
    
    RETURN v_attendance_id;
END;
$$ LANGUAGE plpgsql;

-- Funkcja do potwierdzania wsiadania dziecka
CREATE OR REPLACE FUNCTION confirm_child_pickup(
    p_attendance_id UUID,
    p_stop_id UUID,
    p_method confirmation_method DEFAULT 'manual'
)
RETURNS BOOLEAN AS $$
DECLARE
    v_child_id UUID;
    v_parent_id UUID;
    v_child_name TEXT;
    v_stop_name TEXT;
BEGIN
    -- Pobierz dane dziecka
    SELECT ca.child_id, cr.parent_user_id,
           cr.child_first_name || ' ' || cr.child_last_name,
           s.name
    INTO v_child_id, v_parent_id, v_child_name, v_stop_name
    FROM child_attendance ca
    JOIN child_registrations cr ON ca.child_id = cr.id
    LEFT JOIN stops s ON s.id = p_stop_id
    WHERE ca.id = p_attendance_id;
    
    -- Zaktualizuj obecność
    UPDATE child_attendance
    SET pickup_stop_id = p_stop_id,
        pickup_time = NOW(),
        status = 'picked_up',
        confirmed_by = p_method
    WHERE id = p_attendance_id;
    
    -- Wyślij powiadomienie do rodzica
    PERFORM send_parent_notification(
        v_parent_id,
        v_child_id,
        'child_picked_up',
        'Dziecko wsiadło do autobusu',
        v_child_name || ' wsiadło do autobusu na przystanku ' || v_stop_name,
        jsonb_build_object(
            'stop_name', v_stop_name,
            'pickup_time', NOW()
        )
    );
    
    -- Oznacz że powiadomienie zostało wysłane
    UPDATE child_attendance
    SET parent_notified_pickup = TRUE
    WHERE id = p_attendance_id;
    
    RETURN TRUE;
END;
$$ LANGUAGE plpgsql;

-- Funkcja do potwierdzania wysiadania dziecka
CREATE OR REPLACE FUNCTION confirm_child_dropoff(
    p_attendance_id UUID,
    p_stop_id UUID,
    p_method confirmation_method DEFAULT 'manual'
)
RETURNS BOOLEAN AS $$
DECLARE
    v_child_id UUID;
    v_parent_id UUID;
    v_child_name TEXT;
    v_stop_name TEXT;
BEGIN
    -- Pobierz dane dziecka
    SELECT ca.child_id, cr.parent_user_id,
           cr.child_first_name || ' ' || cr.child_last_name,
           s.name
    INTO v_child_id, v_parent_id, v_child_name, v_stop_name
    FROM child_attendance ca
    JOIN child_registrations cr ON ca.child_id = cr.id
    LEFT JOIN stops s ON s.id = p_stop_id
    WHERE ca.id = p_attendance_id;
    
    -- Zaktualizuj obecność
    UPDATE child_attendance
    SET dropoff_stop_id = p_stop_id,
        dropoff_time = NOW(),
        status = 'dropped_off',
        confirmed_by = p_method
    WHERE id = p_attendance_id;
    
    -- Wyślij powiadomienie do rodzica
    PERFORM send_parent_notification(
        v_parent_id,
        v_child_id,
        'child_dropped_off',
        'Dziecko wysiadło z autobusu',
        v_child_name || ' wysiadło z autobusu na przystanku ' || v_stop_name,
        jsonb_build_object(
            'stop_name', v_stop_name,
            'dropoff_time', NOW()
        )
    );
    
    -- Oznacz że powiadomienie zostało wysłane
    UPDATE child_attendance
    SET parent_notified_dropoff = TRUE
    WHERE id = p_attendance_id;
    
    RETURN TRUE;
END;
$$ LANGUAGE plpgsql;

-- Funkcja do wysyłania powiadomienia do rodzica
CREATE OR REPLACE FUNCTION send_parent_notification(
    p_parent_id UUID,
    p_child_id UUID,
    p_type parent_notification_type,
    p_title VARCHAR(255),
    p_message TEXT,
    p_data JSONB DEFAULT NULL
)
RETURNS UUID AS $$
DECLARE
    v_notification_id UUID;
BEGIN
    INSERT INTO parent_notifications (
        parent_user_id, child_id, notification_type, title, message, data
    ) VALUES (
        p_parent_id, p_child_id, p_type, p_title, p_message, p_data
    ) RETURNING id INTO v_notification_id;
    
    -- W rzeczywistości tutaj byłaby też wysyłka push notification
    -- przez Firebase Cloud Messaging lub podobną usługę
    
    RETURN v_notification_id;
END;
$$ LANGUAGE plpgsql;

-- Funkcja do oznaczania powiadomienia jako przeczytane
CREATE OR REPLACE FUNCTION mark_notification_read(p_notification_id UUID)
RETURNS BOOLEAN AS $$
BEGIN
    UPDATE parent_notifications
    SET read_at = NOW()
    WHERE id = p_notification_id AND read_at IS NULL;
    
    RETURN FOUND;
END;
$$ LANGUAGE plpgsql;

-- Funkcja do oznaczania dziecka jako nieobecne
CREATE OR REPLACE FUNCTION mark_child_absent(
    p_child_id UUID,
    p_date DATE DEFAULT CURRENT_DATE
)
RETURNS BOOLEAN AS $$
BEGIN
    UPDATE child_attendance
    SET status = 'absent'
    WHERE child_id = p_child_id AND date = p_date;
    
    RETURN FOUND;
END;
$$ LANGUAGE plpgsql;

-- ============================================
-- WIDOKI (VIEWS)
-- ============================================

-- Widok dzieci z danymi rodzica i trasy
CREATE VIEW children_view AS
SELECT 
    cr.*,
    CONCAT(cr.child_first_name, ' ', cr.child_last_name) as child_full_name,
    u.first_name as parent_first_name,
    u.last_name as parent_last_name,
    u.email as parent_email,
    u.phone as parent_phone,
    r.name as route_name,
    r.number as route_number,
    r.color as route_color,
    pickup.name as pickup_stop_name,
    dropoff.name as dropoff_stop_name
FROM child_registrations cr
JOIN users u ON cr.parent_user_id = u.id
LEFT JOIN routes r ON cr.assigned_route_id = r.id
LEFT JOIN stops pickup ON cr.pickup_stop_id = pickup.id
LEFT JOIN stops dropoff ON cr.dropoff_stop_id = dropoff.id;

-- Widok dzisiejszej obecności dla kierowcy
CREATE VIEW today_attendance_view AS
SELECT 
    ca.*,
    CONCAT(cr.child_first_name, ' ', cr.child_last_name) as child_full_name,
    cr.child_birth_date,
    cr.qr_code,
    cr.photo_url,
    cr.school_name,
    pickup.name as pickup_stop_name,
    dropoff.name as dropoff_stop_name,
    r.name as route_name,
    r.number as route_number,
    v.registration_number as vehicle_registration
FROM child_attendance ca
JOIN child_registrations cr ON ca.child_id = cr.id
LEFT JOIN stops pickup ON ca.pickup_stop_id = pickup.id
LEFT JOIN stops dropoff ON ca.dropoff_stop_id = dropoff.id
LEFT JOIN routes r ON ca.route_id = r.id
LEFT JOIN vehicles v ON ca.vehicle_id = v.id
WHERE ca.date = CURRENT_DATE;

-- Widek obecności z historią dla rodzica
CREATE VIEW child_attendance_history_view AS
SELECT 
    ca.*,
    CONCAT(cr.child_first_name, ' ', cr.child_last_name) as child_full_name,
    pickup.name as pickup_stop_name,
    dropoff.name as dropoff_stop_name,
    r.name as route_name,
    r.number as route_number
FROM child_attendance ca
JOIN child_registrations cr ON ca.child_id = cr.id
LEFT JOIN stops pickup ON ca.pickup_stop_id = pickup.id
LEFT JOIN stops dropoff ON ca.dropoff_stop_id = dropoff.id
LEFT JOIN routes r ON ca.route_id = r.id;

-- Widok nieprzeczytanych powiadomień
CREATE VIEW unread_notifications_view AS
SELECT 
    pn.*,
    CONCAT(cr.child_first_name, ' ', cr.child_last_name) as child_full_name
FROM parent_notifications pn
LEFT JOIN child_registrations cr ON pn.child_id = cr.id
WHERE pn.read_at IS NULL;

-- ============================================
-- KOMENTARZE
-- ============================================

COMMENT ON TABLE child_registrations IS 'Rejestracje dzieci w systemie transportu szkolnego';
COMMENT ON TABLE child_attendance IS 'Dzienna obecność dzieci w transporcie';
COMMENT ON TABLE parent_notifications IS 'Powiadomienia wysyłane do rodziców';

COMMENT ON COLUMN child_registrations.qr_code IS 'Unikalny kod QR do skanowania przy wsiadaniu';
COMMENT ON COLUMN child_attendance.pickup_time IS 'Czas faktycznego wsiadania';
COMMENT ON COLUMN child_attendance.dropoff_time IS 'Czas faktycznego wysiadania';
COMMENT ON COLUMN parent_notifications.data IS 'Dodatkowe dane JSON (np. lokalizacja, czas)';
