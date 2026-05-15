CREATE TABLE webhook_endpoints (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,
    url TEXT NOT NULL,
    secret_env TEXT,
    subscribed_event_kinds_json TEXT NOT NULL,
    timeout_ms INTEGER NOT NULL,
    max_attempts INTEGER NOT NULL,
    status TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE INDEX webhook_endpoints_status_idx
    ON webhook_endpoints(status);

CREATE TABLE webhook_delivery_attempts (
    id TEXT PRIMARY KEY NOT NULL,
    endpoint_id TEXT NOT NULL REFERENCES webhook_endpoints(id) ON DELETE CASCADE,
    event_id TEXT NOT NULL REFERENCES event_outbox(id) ON DELETE CASCADE,
    attempt_number INTEGER NOT NULL,
    status TEXT NOT NULL,
    http_status INTEGER,
    error TEXT,
    requested_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    completed_at TEXT,
    next_retry_at TEXT,
    UNIQUE(endpoint_id, event_id, attempt_number)
);

CREATE INDEX webhook_delivery_attempts_event_idx
    ON webhook_delivery_attempts(event_id, requested_at);

CREATE INDEX webhook_delivery_attempts_endpoint_idx
    ON webhook_delivery_attempts(endpoint_id, requested_at);

CREATE INDEX webhook_delivery_attempts_status_idx
    ON webhook_delivery_attempts(status, next_retry_at);
