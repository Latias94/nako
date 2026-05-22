CREATE TABLE managed_import_promotion_applies (
    id TEXT PRIMARY KEY NOT NULL,
    artifact_id TEXT NOT NULL REFERENCES managed_import_artifacts(id) ON DELETE CASCADE,
    target_library_id TEXT NOT NULL REFERENCES libraries(id) ON DELETE CASCADE,
    requested_by TEXT NOT NULL,
    idempotency_key TEXT NOT NULL,
    operation_kind TEXT NOT NULL,
    source_artifact_uri TEXT,
    destination_locator TEXT NOT NULL,
    accepted_plan_json TEXT NOT NULL,
    accepted_warnings_json TEXT,
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
    CHECK (length(destination_locator) > 0),
    CHECK (length(accepted_plan_json) > 0)
);

CREATE INDEX managed_import_promotion_applies_artifact_idx
    ON managed_import_promotion_applies(artifact_id, updated_at_ms DESC, id);

CREATE INDEX managed_import_promotion_applies_library_state_idx
    ON managed_import_promotion_applies(target_library_id, state, updated_at_ms DESC, id);
