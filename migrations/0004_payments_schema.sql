-- Migration: Create payments table
-- Created: 2026-03-26

-- Create enum types
CREATE TYPE payment_status AS ENUM ('pending', 'succeeded', 'failed', 'refunded', 'cancelled');
CREATE TYPE payment_method AS ENUM ('card', 'blik', 'bank_transfer', 'apple_pay', 'google_pay');

-- Create payments table
CREATE TABLE payments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    ticket_id UUID REFERENCES tickets(id) ON DELETE SET NULL,
    stripe_payment_intent_id VARCHAR(255),
    stripe_customer_id VARCHAR(255),
    amount INTEGER NOT NULL, -- Amount in cents
    currency VARCHAR(3) NOT NULL DEFAULT 'PLN',
    status payment_status NOT NULL DEFAULT 'pending',
    payment_method payment_method,
    description TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    metadata JSONB,

    -- Constraints
    CONSTRAINT valid_amount CHECK (amount > 0),
    CONSTRAINT valid_currency CHECK (currency IN ('PLN', 'EUR', 'USD'))
);

-- Create indexes
CREATE INDEX idx_payments_user_id ON payments(user_id);
CREATE INDEX idx_payments_status ON payments(status);
CREATE INDEX idx_payments_stripe_intent ON payments(stripe_payment_intent_id);
CREATE INDEX idx_payments_created_at ON payments(created_at DESC);
CREATE INDEX idx_payments_user_status ON payments(user_id, status);

-- Create trigger for updated_at
CREATE OR REPLACE FUNCTION update_payments_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER payments_updated_at
    BEFORE UPDATE ON payments
    FOR EACH ROW
    EXECUTE FUNCTION update_payments_updated_at();

-- Add comments
COMMENT ON TABLE payments IS 'Payment transactions';
COMMENT ON COLUMN payments.stripe_payment_intent_id IS 'Stripe PaymentIntent ID';
COMMENT ON COLUMN payments.stripe_customer_id IS 'Stripe Customer ID';
