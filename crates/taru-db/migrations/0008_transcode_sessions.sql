CREATE TABLE transcode_sessions (
    id TEXT PRIMARY KEY NOT NULL,
    source_id TEXT NOT NULL REFERENCES media_sources(id) ON DELETE CASCADE,
    kind TEXT NOT NULL,
    request_key TEXT NOT NULL,
    output_path TEXT NOT NULL,
    state TEXT NOT NULL,
    failure_category TEXT,
    failure_message TEXT,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    started_at TEXT,
    completed_at TEXT
);

CREATE INDEX transcode_sessions_source_idx
    ON transcode_sessions(source_id);

CREATE INDEX transcode_sessions_request_idx
    ON transcode_sessions(source_id, kind, request_key, updated_at);

CREATE INDEX transcode_sessions_state_idx
    ON transcode_sessions(state);

CREATE UNIQUE INDEX transcode_sessions_active_request_idx
    ON transcode_sessions(source_id, kind, request_key)
    WHERE state IN ('planned', 'starting', 'running', 'cancel_requested');
