CREATE TABLE ticket_validation_attempts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    controller_id UUID NOT NULL REFERENCES users(id),
    ticket_id UUID REFERENCES tickets(id),
    qr_fingerprint TEXT NOT NULL,
    result TEXT NOT NULL CHECK (result IN (
        'valid', 'used', 'expired', 'invalid_signature', 'wrong_tenant', 'forbidden'
    )),
    vehicle_id UUID REFERENCES vehicles(id),
    latitude DOUBLE PRECISION,
    longitude DOUBLE PRECISION,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CHECK (char_length(qr_fingerprint) = 64),
    CHECK (latitude IS NULL OR latitude BETWEEN -90 AND 90),
    CHECK (longitude IS NULL OR longitude BETWEEN -180 AND 180)
);

CREATE INDEX ticket_validation_attempts_controller_created_idx
    ON ticket_validation_attempts (controller_id, created_at DESC);

CREATE INDEX ticket_validation_attempts_ticket_created_idx
    ON ticket_validation_attempts (ticket_id, created_at DESC)
    WHERE ticket_id IS NOT NULL;
