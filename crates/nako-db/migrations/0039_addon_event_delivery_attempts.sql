CREATE TABLE addon_event_delivery_attempts (
    id TEXT PRIMARY KEY NOT NULL,
    addon_id TEXT NOT NULL REFERENCES addon_registrations(id) ON DELETE CASCADE,
    event_id TEXT NOT NULL REFERENCES event_outbox(id) ON DELETE CASCADE,
    declaration_id TEXT NOT NULL,
    attempt_number INTEGER NOT NULL,
    status TEXT NOT NULL,
    http_status INTEGER,
    error TEXT,
    requested_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    completed_at TEXT,
    next_retry_at TEXT,
    UNIQUE(addon_id, event_id, declaration_id, attempt_number)
);

CREATE INDEX addon_event_delivery_attempts_event_idx
    ON addon_event_delivery_attempts(event_id, requested_at);

CREATE INDEX addon_event_delivery_attempts_addon_idx
    ON addon_event_delivery_attempts(addon_id, event_id, declaration_id, requested_at);

CREATE INDEX addon_event_delivery_attempts_status_idx
    ON addon_event_delivery_attempts(status, next_retry_at);
