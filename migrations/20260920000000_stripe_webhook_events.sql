CREATE TABLE stripe_webhook_events (
    event_id TEXT PRIMARY KEY,
    event_type TEXT NOT NULL,
    stripe_payment_intent_id TEXT,
    api_version TEXT,
    processed_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX stripe_webhook_events_payment_intent_idx
    ON stripe_webhook_events (stripe_payment_intent_id);
