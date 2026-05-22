CREATE TABLE nfo_sidecar_applies (
    id TEXT PRIMARY KEY NOT NULL,
    target_library_id TEXT NOT NULL REFERENCES libraries(id) ON DELETE CASCADE,
    media_item_id TEXT NOT NULL REFERENCES media_items(id) ON DELETE CASCADE,
    media_source_id TEXT REFERENCES media_sources(id) ON DELETE SET NULL,
    requested_by TEXT NOT NULL,
    idempotency_key TEXT NOT NULL,
    operation_kind TEXT NOT NULL,
    sidecar_locator TEXT NOT NULL,
    accepted_preview_json TEXT NOT NULL,
    accepted_warnings_json TEXT,
    policy_version TEXT NOT NULL,
    state TEXT NOT NULL,
    outcome_json TEXT,
    safe_error_code TEXT,
    safe_message TEXT,
    created_at_ms INTEGER NOT NULL,
    updated_at_ms INTEGER NOT NULL,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    UNIQUE(target_library_id, idempotency_key),
    CHECK (length(idempotency_key) > 0),
    CHECK (length(sidecar_locator) > 0),
    CHECK (length(accepted_preview_json) > 0),
    CHECK (length(policy_version) > 0)
);

CREATE INDEX nfo_sidecar_applies_item_idx
    ON nfo_sidecar_applies(media_item_id, updated_at_ms DESC, id);

CREATE INDEX nfo_sidecar_applies_source_idx
    ON nfo_sidecar_applies(media_source_id, updated_at_ms DESC, id);

CREATE INDEX nfo_sidecar_applies_library_state_idx
    ON nfo_sidecar_applies(target_library_id, state, updated_at_ms DESC, id);
