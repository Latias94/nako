CREATE TABLE IF NOT EXISTS addon_event_delivery_attempts (
    id uuid PRIMARY KEY NOT NULL,
    addon_id uuid NOT NULL REFERENCES addon_registrations(id) ON DELETE CASCADE,
    event_id uuid NOT NULL REFERENCES event_outbox(id) ON DELETE CASCADE,
    declaration_id text NOT NULL,
    attempt_number bigint NOT NULL,
    status text NOT NULL,
    http_status bigint,
    error text,
    requested_at timestamptz NOT NULL DEFAULT statement_timestamp(),
    completed_at timestamptz,
    next_retry_at text,
    UNIQUE(addon_id, event_id, declaration_id, attempt_number)
);

CREATE INDEX IF NOT EXISTS addon_event_delivery_attempts_event_idx
    ON addon_event_delivery_attempts(event_id, requested_at);

CREATE INDEX IF NOT EXISTS addon_event_delivery_attempts_addon_idx
    ON addon_event_delivery_attempts(addon_id, event_id, declaration_id, requested_at);

CREATE INDEX IF NOT EXISTS addon_event_delivery_attempts_status_idx
    ON addon_event_delivery_attempts(status, next_retry_at);
