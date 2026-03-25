-- Migration: Create tickets table
-- Created: 2026-03-26

-- Create enum types
CREATE TYPE ticket_type AS ENUM ('single', 'monthly', 'weekly', 'discounted');
CREATE TYPE ticket_status AS ENUM ('active', 'used', 'expired', 'cancelled');

-- Create tickets table
CREATE TABLE tickets (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    ticket_type ticket_type NOT NULL DEFAULT 'single',
    status ticket_status NOT NULL DEFAULT 'active',
    qr_code TEXT NOT NULL UNIQUE,
    price INTEGER NOT NULL, -- Price in cents (e.g., 1000 = 10.00 PLN)
    currency VARCHAR(3) NOT NULL DEFAULT 'PLN',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    valid_until TIMESTAMPTZ NOT NULL,
    used_at TIMESTAMPTZ,
    route_id UUID REFERENCES routes(id) ON DELETE SET NULL,
    start_stop_id UUID REFERENCES stops(id) ON DELETE SET NULL,
    end_stop_id UUID REFERENCES stops(id) ON DELETE SET NULL,
    metadata JSONB,

    -- Constraints
    CONSTRAINT valid_price CHECK (price >= 0),
    CONSTRAINT valid_dates CHECK (valid_until > created_at)
);

-- Create indexes for common queries
CREATE INDEX idx_tickets_user_id ON tickets(user_id);
CREATE INDEX idx_tickets_status ON tickets(status);
CREATE INDEX idx_tickets_qr_code ON tickets(qr_code);
CREATE INDEX idx_tickets_valid_until ON tickets(valid_until);
CREATE INDEX idx_tickets_created_at ON tickets(created_at DESC);

-- Create index for active tickets lookup
CREATE INDEX idx_tickets_active ON tickets(user_id, status) WHERE status = 'active';

-- Create table for ticket validations (scan history)
CREATE TABLE ticket_validations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    ticket_id UUID NOT NULL REFERENCES tickets(id) ON DELETE CASCADE,
    validated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    vehicle_id UUID REFERENCES vehicles(id) ON DELETE SET NULL,
    driver_id UUID REFERENCES users(id) ON DELETE SET NULL,
    latitude DOUBLE PRECISION,
    longitude DOUBLE PRECISION,
    is_valid BOOLEAN NOT NULL,
    error_message TEXT,

    -- Constraints
    CONSTRAINT valid_latitude CHECK (latitude >= -90 AND latitude <= 90),
    CONSTRAINT valid_longitude CHECK (longitude >= -180 AND longitude <= 180)
);

-- Create indexes for validations
CREATE INDEX idx_ticket_validations_ticket_id ON ticket_validations(ticket_id);
CREATE INDEX idx_ticket_validations_validated_at ON ticket_validations(validated_at DESC);
CREATE INDEX idx_ticket_validations_vehicle ON ticket_validations(vehicle_id);

-- Add comment
COMMENT ON TABLE tickets IS 'User tickets with QR codes';
COMMENT ON TABLE ticket_validations IS 'History of ticket validations (scans)';
