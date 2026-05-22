CREATE TABLE event_outbox (
    id TEXT PRIMARY KEY NOT NULL,
    kind TEXT NOT NULL,
    subject_kind TEXT NOT NULL,
    subject_id TEXT NOT NULL,
    library_id TEXT REFERENCES libraries(id) ON DELETE SET NULL,
    source_id TEXT REFERENCES media_sources(id) ON DELETE SET NULL,
    idempotency_key TEXT NOT NULL,
    payload_json TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'pending',
    attempts INTEGER NOT NULL DEFAULT 0,
    last_error TEXT,
    occurred_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    next_attempt_at TEXT,
    UNIQUE(kind, idempotency_key)
);

CREATE INDEX event_outbox_status_idx
    ON event_outbox(status, occurred_at);

CREATE INDEX event_outbox_subject_idx
    ON event_outbox(subject_kind, subject_id, occurred_at);

CREATE INDEX event_outbox_library_idx
    ON event_outbox(library_id, occurred_at);

CREATE INDEX event_outbox_source_idx
    ON event_outbox(source_id, occurred_at);
