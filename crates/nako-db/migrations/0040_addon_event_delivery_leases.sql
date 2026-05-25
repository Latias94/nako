ALTER TABLE addon_event_delivery_attempts
    ADD COLUMN lease_expires_at TEXT;

CREATE INDEX addon_event_delivery_attempts_lease_idx
    ON addon_event_delivery_attempts(status, lease_expires_at);
