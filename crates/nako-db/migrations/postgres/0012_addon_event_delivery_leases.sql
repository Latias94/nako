ALTER TABLE addon_event_delivery_attempts
    ADD COLUMN IF NOT EXISTS lease_expires_at text;

CREATE INDEX IF NOT EXISTS addon_event_delivery_attempts_lease_idx
    ON addon_event_delivery_attempts(status, lease_expires_at);
