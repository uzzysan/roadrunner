-- Sample data for RoadRunner - Warsaw public transport stops and routes
-- This data is for testing purposes and uses approximate coordinates

-- ============================================
-- PRZYSTANKI (Stops)
-- ============================================

INSERT INTO stops (name, location, address, amenities, is_active) VALUES
-- Centrum
('Plac Defilad', 'SRID=4326;POINT(21.0083 52.2319)', 'Plac Defilad, Warszawa', ARRAY['shelter', 'bench', 'lighting', 'display'], true),
('Dworzec Centralny', 'SRID=4326;POINT(21.0032 52.2288)', 'Aleje Jerozolimskie 54, Warszawa', ARRAY['shelter', 'bench', 'lighting', 'monitoring', 'ticket_machine', 'display'], true),
('Metro Świętokrzyska', 'SRID=4326;POINT(21.0115 52.2351)', 'ul. Świętokrzyska, Warszawa', ARRAY['shelter', 'bench', 'lighting', 'display'], true),
('Plac Bankowy', 'SRID=4326;POINT(21.0025 52.2420)', 'Plac Bankowy, Warszawa', ARRAY['shelter', 'bench', 'lighting', 'monitoring'], true),
('Uniwersytet', 'SRID=4326;POINT(21.0167 52.2396)', 'Krakowskie Przedmieście, Warszawa', ARRAY['shelter', 'bench', 'lighting'], true),

-- Mokotów
('Mokotów', 'SRID=4326;POINT(21.0175 52.1931)', 'Aleja Niepodległości, Warszawa', ARRAY['shelter', 'bench', 'lighting'], true),
('Wierzbno', 'SRID=4326;POINT(21.0125 52.1892)', 'Aleja Niepodległości, Warszawa', ARRAY['shelter', 'bench', 'lighting', 'display'], true),
('Służew', 'SRID=4326;POINT(21.0267 52.1728)', 'ul. Wałbrzyska, Warszawa', ARRAY['shelter', 'bench', 'lighting'], true),
('Ursynów', 'SRID=4326;POINT(21.0325 52.1578)', 'Aleja Komisji Edukacji Narodowej, Warszawa', ARRAY['shelter', 'bench', 'lighting', 'monitoring'], true),
('Natolin', 'SRID=4326;POINT(21.0417 52.1456)', 'Aleja Komisji Edukacji Narodowej, Warszawa', ARRAY['shelter', 'bench', 'lighting'], true),

-- Żoliborz
('Plac Wilsona', 'SRID=4326;POINT(20.9869 52.2694)', 'Plac Wilsona, Warszawa', ARRAY['shelter', 'bench', 'lighting', 'display'], true),
('Metro Marymont', 'SRID=4326;POINT(20.9714 52.2714)', 'ul. Słowackiego, Warszawa', ARRAY['shelter', 'bench', 'lighting'], true),
('Bielany', 'SRID=4326;POINT(20.9350 52.2900)', 'ul. Kasprowicza, Warszawa', ARRAY['shelter', 'bench', 'lighting'], true),
('Młociny', 'SRID=4326;POINT(20.9167 52.3000)', 'ul. Kasprowicza, Warszawa', ARRAY['shelter', 'bench', 'lighting', 'monitoring', 'ticket_machine'], true),

-- Praga
('Dworzec Wileński', 'SRID=4326;POINT(21.0358 52.2542)', 'ul. Targowa, Warszawa', ARRAY['shelter', 'bench', 'lighting', 'monitoring', 'display'], true),
('Metro Stadion', 'SRID=4326;POINT(21.0417 52.2394)', 'ul. Sokola, Warszawa', ARRAY['shelter', 'bench', 'lighting'], true),
('Gocław', 'SRID=4326;POINT(21.0833 52.2167)', 'ul. Fieldorfa, Warszawa', ARRAY['shelter', 'bench', 'lighting'], true),
('Grochów', 'SRID=4326;POINT(21.0833 52.2333)', 'ul. Grochowska, Warszawa', ARRAY['shelter', 'bench', 'lighting'], true),

-- Wola
('Rondo Daszyńskiego', 'SRID=4326;POINT(20.9833 52.2306)', 'ul. Prosta, Warszawa', ARRAY['shelter', 'bench', 'lighting', 'display'], true),
('Kasprzaka', 'SRID=4326;POINT(20.9667 52.2250)', 'ul. Kasprzaka, Warszawa', ARRAY['shelter', 'bench', 'lighting'], true),
('Czyste', 'SRID=4326;POINT(20.9583 52.2250)', 'ul. Wolska, Warszawa', ARRAY['shelter', 'bench', 'lighting'], true),

-- Ochota
('Rakowiec', 'SRID=4326;POINT(20.9833 52.2000)', 'ul. Grójecka, Warszawa', ARRAY['shelter', 'bench', 'lighting'], true),
('Okęcie', 'SRID=4326;POINT(20.9667 52.1667)', 'ul. Żwirki i Wigury, Warszawa', ARRAY['shelter', 'bench', 'lighting', 'monitoring'], true),
('Szpital Banacha', 'SRID=4326;POINT(20.9833 52.2083)', 'ul. Banacha, Warszawa', ARRAY['shelter', 'bench', 'lighting', 'accessibility'], true);

-- ============================================
-- LINIE (Routes)
-- ============================================

INSERT INTO routes (name, number, description, color, is_active) VALUES
('Linia 175', '175', 'Plac Piłsudskiego - Port Lotniczy im. Chopina', '#2563EB', true),
('Linia 128', '128', 'Plac Trzech Krzyży - Jeziorki', '#EF4444', true),
('Linia 504', '504', 'Dworzec Centralny - Bemowo', '#10B981', true),
('Linia 523', '523', 'Dworzec Centralny - Nowe Bemowo', '#F59E0B', true),
('Linia 180', '180', 'Chomiczówka - Metro Politechnika', '#8B5CF6', true),
('Linia 159', '159', 'Sadyba - Chomiczówka', '#EC4899', true),
('Linia 365', '365', 'Ursynów Południowy - Regulska', '#06B6D4', true),
('Linia 217', '217', 'Metro Wilanowska - Natolin Północny', '#84CC16', true);

-- ============================================
-- PRZYSTANKI W TRASACH (Route Stops)
-- ============================================

-- Linia 175: Plac Piłsudskiego - Port Lotniczy
INSERT INTO route_stops (route_id, stop_id, stop_order, is_optional) 
SELECT 
    r.id as route_id,
    s.id as stop_id,
    ord.stop_order,
    ord.is_optional
FROM routes r
CROSS JOIN (VALUES 
    ('Plac Defilad', 1, false),
    ('Dworzec Centralny', 2, false),
    ('Metro Świętokrzyska', 3, false),
    ('Plac Bankowy', 4, false),
    ('Rondo Daszyńskiego', 5, false),
    ('Rakowiec', 6, false),
    ('Okęcie', 7, false)
) AS ord(name, stop_order, is_optional)
JOIN stops s ON s.name = ord.name
WHERE r.number = '175';

-- Linia 128: Plac Trzech Krzyży - Jeziorki
INSERT INTO route_stops (route_id, stop_id, stop_order, is_optional)
SELECT 
    r.id as route_id,
    s.id as stop_id,
    ord.stop_order,
    ord.is_optional
FROM routes r
CROSS JOIN (VALUES 
    ('Uniwersytet', 1, false),
    ('Mokotów', 2, false),
    ('Wierzbno', 3, false),
    ('Służew', 4, false),
    ('Ursynów', 5, false),
    ('Natolin', 6, false)
) AS ord(name, stop_order, is_optional)
JOIN stops s ON s.name = ord.name
WHERE r.number = '128';

-- Linia 504: Dworzec Centralny - Bemowo
INSERT INTO route_stops (route_id, stop_id, stop_order, is_optional)
SELECT 
    r.id as route_id,
    s.id as stop_id,
    ord.stop_order,
    ord.is_optional
FROM routes r
CROSS JOIN (VALUES 
    ('Dworzec Centralny', 1, false),
    ('Rondo Daszyńskiego', 2, false),
    ('Kasprzaka', 3, false),
    ('Czyste', 4, false)
) AS ord(name, stop_order, is_optional)
JOIN stops s ON s.name = ord.name
WHERE r.number = '504';

-- Linia 180: Chomiczówka - Metro Politechnika
INSERT INTO route_stops (route_id, stop_id, stop_order, is_optional)
SELECT 
    r.id as route_id,
    s.id as stop_id,
    ord.stop_order,
    ord.is_optional
FROM routes r
CROSS JOIN (VALUES 
    ('Młociny', 1, false),
    ('Bielany', 2, false),
    ('Metro Marymont', 3, false),
    ('Plac Wilsona', 4, false),
    ('Plac Bankowy', 5, false),
    ('Uniwersytet', 6, false)
) AS ord(name, stop_order, is_optional)
JOIN stops s ON s.name = ord.name
WHERE r.number = '180';

-- Linia 159: Sadyba - Chomiczówka
INSERT INTO route_stops (route_id, stop_id, stop_order, is_optional)
SELECT 
    r.id as route_id,
    s.id as stop_id,
    ord.stop_order,
    ord.is_optional
FROM routes r
CROSS JOIN (VALUES 
    ('Mokotów', 1, false),
    ('Wierzbno', 2, false),
    ('Metro Świętokrzyska', 3, false),
    ('Plac Wilsona', 4, false),
    ('Bielany', 5, false)
) AS ord(name, stop_order, is_optional)
JOIN stops s ON s.name = ord.name
WHERE r.number = '159';

-- Linia 217: Metro Wilanowska - Natolin Północny
INSERT INTO route_stops (route_id, stop_id, stop_order, is_optional)
SELECT 
    r.id as route_id,
    s.id as stop_id,
    ord.stop_order,
    ord.is_optional
FROM routes r
CROSS JOIN (VALUES 
    ('Mokotów', 1, false),
    ('Służew', 2, false),
    ('Ursynów', 3, false),
    ('Natolin', 4, false)
) AS ord(name, stop_order, is_optional)
JOIN stops s ON s.name = ord.name
WHERE r.number = '217';

-- ============================================
-- ROZKŁADY JAZDY (Schedules)
-- ============================================

-- Funkcja pomocnicza do generowania rozkładu
CREATE OR REPLACE FUNCTION generate_schedule(
    p_route_number TEXT,
    p_stop_name TEXT,
    p_day_type day_type,
    p_first_time TIME,
    p_last_time TIME,
    p_interval_minutes INT
) RETURNS VOID AS $$
DECLARE
    v_route_id UUID;
    v_stop_id UUID;
    v_current_time TIME;
BEGIN
    SELECT id INTO v_route_id FROM routes WHERE number = p_route_number;
    SELECT id INTO v_stop_id FROM stops WHERE name = p_stop_name;
    
    IF v_route_id IS NULL OR v_stop_id IS NULL THEN
        RETURN;
    END IF;
    
    v_current_time := p_first_time;
    
    WHILE v_current_time <= p_last_time LOOP
        INSERT INTO schedules (route_id, stop_id, arrival_time, departure_time, day_type, is_active)
        VALUES (v_route_id, v_stop_id, v_current_time, v_current_time, p_day_type, true)
        ON CONFLICT DO NOTHING;
        
        v_current_time := v_current_time + (p_interval_minutes || ' minutes')::INTERVAL;
    END LOOP;
END;
$$ LANGUAGE plpgsql;

-- Linia 175 - Plac Defilad (co 15 min w dni robocze)
SELECT generate_schedule('175', 'Plac Defilad', 'weekday', '05:00'::TIME, '23:00'::TIME, 15);
SELECT generate_schedule('175', 'Plac Defilad', 'saturday', '06:00'::TIME, '23:00'::TIME, 20);
SELECT generate_schedule('175', 'Plac Defilad', 'sunday', '07:00'::TIME, '22:00'::TIME, 30);

-- Linia 175 - Dworzec Centralny
SELECT generate_schedule('175', 'Dworzec Centralny', 'weekday', '05:05'::TIME, '23:05'::TIME, 15);
SELECT generate_schedule('175', 'Dworzec Centralny', 'saturday', '06:05'::TIME, '23:05'::TIME, 20);
SELECT generate_schedule('175', 'Dworzec Centralny', 'sunday', '07:05'::TIME, '22:05'::TIME, 30);

-- Linia 128 - Uniwersytet (co 10 min w dni robocze)
SELECT generate_schedule('128', 'Uniwersytet', 'weekday', '05:30'::TIME, '23:30'::TIME, 10);
SELECT generate_schedule('128', 'Uniwersytet', 'saturday', '06:30'::TIME, '23:30'::TIME, 15);
SELECT generate_schedule('128', 'Uniwersytet', 'sunday', '07:30'::TIME, '22:30'::TIME, 20);

-- Linia 128 - Ursynów
SELECT generate_schedule('128', 'Ursynów', 'weekday', '05:50'::TIME, '23:50'::TIME, 10);
SELECT generate_schedule('128', 'Ursynów', 'saturday', '06:50'::TIME, '23:50'::TIME, 15);
SELECT generate_schedule('128', 'Ursynów', 'sunday', '07:50'::TIME, '22:50'::TIME, 20);

-- Linia 504 - Dworzec Centralny (co 8 min w szczycie)
SELECT generate_schedule('504', 'Dworzec Centralny', 'weekday', '05:00'::TIME, '09:00'::TIME, 8);
SELECT generate_schedule('504', 'Dworzec Centralny', 'weekday', '14:00'::TIME, '19:00'::TIME, 8);
SELECT generate_schedule('504', 'Dworzec Centralny', 'weekday', '09:00'::TIME, '14:00'::TIME, 12);
SELECT generate_schedule('504', 'Dworzec Centralny', 'weekday', '19:00'::TIME, '23:00'::TIME, 15);
SELECT generate_schedule('504', 'Dworzec Centralny', 'saturday', '06:00'::TIME, '23:00'::TIME, 15);
SELECT generate_schedule('504', 'Dworzec Centralny', 'sunday', '07:00'::TIME, '22:00'::TIME, 20);

-- Linia 180 - Młociny (co 12 min)
SELECT generate_schedule('180', 'Młociny', 'weekday', '05:00'::TIME, '23:00'::TIME, 12);
SELECT generate_schedule('180', 'Młociny', 'saturday', '06:00'::TIME, '23:00'::TIME, 15);
SELECT generate_schedule('180', 'Młociny', 'sunday', '07:00'::TIME, '22:00'::TIME, 20);

-- Linia 180 - Plac Wilsona
SELECT generate_schedule('180', 'Plac Wilsona', 'weekday', '05:15'::TIME, '23:15'::TIME, 12);
SELECT generate_schedule('180', 'Plac Wilsona', 'saturday', '06:15'::TIME, '23:15'::TIME, 15);
SELECT generate_schedule('180', 'Plac Wilsona', 'sunday', '07:15'::TIME, '22:15'::TIME, 20);

-- Linia 217 - Natolin (co 20 min)
SELECT generate_schedule('217', 'Natolin', 'weekday', '05:00'::TIME, '23:00'::TIME, 20);
SELECT generate_schedule('217', 'Natolin', 'saturday', '06:00'::TIME, '23:00'::TIME, 25);
SELECT generate_schedule('217', 'Natolin', 'sunday', '07:00'::TIME, '22:00'::TIME, 30);

-- Usuń funkcję pomocniczą
DROP FUNCTION IF EXISTS generate_schedule(TEXT, TEXT, day_type, TIME, TIME, INT);

-- ============================================
-- STATYSTYKI
-- ============================================

-- Sprawdź liczbę przystanków
SELECT 'Przystanki' as tabela, COUNT(*) as liczba FROM stops WHERE is_active = true
UNION ALL
SELECT 'Linie', COUNT(*) FROM routes WHERE is_active = true
UNION ALL
SELECT 'Przystanki w trasach', COUNT(*) FROM route_stops
UNION ALL
SELECT 'Rozkłady jazdy', COUNT(*) FROM schedules WHERE is_active = true;
