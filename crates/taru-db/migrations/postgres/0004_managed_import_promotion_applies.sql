CREATE TABLE IF NOT EXISTS managed_import_promotion_applies (
    id uuid PRIMARY KEY NOT NULL,
    artifact_id uuid NOT NULL REFERENCES managed_import_artifacts(id) ON DELETE CASCADE,
    target_library_id uuid NOT NULL REFERENCES libraries(id) ON DELETE CASCADE,
    requested_by text NOT NULL,
    idempotency_key text NOT NULL,
    operation_kind text NOT NULL,
    source_artifact_uri text,
    destination_locator text NOT NULL,
    accepted_plan_json text NOT NULL,
    accepted_warnings_json text,
    state text NOT NULL,
    outcome_json text,
    safe_error_code text,
    safe_message text,
    created_at_ms bigint NOT NULL,
    updated_at_ms bigint NOT NULL,
    created_at timestamptz NOT NULL DEFAULT statement_timestamp(),
    updated_at timestamptz NOT NULL DEFAULT statement_timestamp(),
    UNIQUE(target_library_id, idempotency_key),
    CHECK (length(idempotency_key) > 0),
    CHECK (length(destination_locator) > 0),
    CHECK (length(accepted_plan_json) > 0)
);

CREATE INDEX IF NOT EXISTS managed_import_promotion_applies_artifact_idx
    ON managed_import_promotion_applies(artifact_id, updated_at_ms DESC, id);

CREATE INDEX IF NOT EXISTS managed_import_promotion_applies_library_state_idx
    ON managed_import_promotion_applies(target_library_id, state, updated_at_ms DESC, id);
