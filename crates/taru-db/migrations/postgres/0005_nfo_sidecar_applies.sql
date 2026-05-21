CREATE TABLE IF NOT EXISTS nfo_sidecar_applies (
    id uuid PRIMARY KEY NOT NULL,
    target_library_id uuid NOT NULL REFERENCES libraries(id) ON DELETE CASCADE,
    media_item_id uuid NOT NULL REFERENCES media_items(id) ON DELETE CASCADE,
    media_source_id uuid REFERENCES media_sources(id) ON DELETE SET NULL,
    requested_by text NOT NULL,
    idempotency_key text NOT NULL,
    operation_kind text NOT NULL,
    sidecar_locator text NOT NULL,
    accepted_preview_json text NOT NULL,
    accepted_warnings_json text,
    policy_version text NOT NULL,
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
    CHECK (length(sidecar_locator) > 0),
    CHECK (length(accepted_preview_json) > 0),
    CHECK (length(policy_version) > 0)
);

CREATE INDEX IF NOT EXISTS nfo_sidecar_applies_item_idx
    ON nfo_sidecar_applies(media_item_id, updated_at_ms DESC, id);

CREATE INDEX IF NOT EXISTS nfo_sidecar_applies_source_idx
    ON nfo_sidecar_applies(media_source_id, updated_at_ms DESC, id);

CREATE INDEX IF NOT EXISTS nfo_sidecar_applies_library_state_idx
    ON nfo_sidecar_applies(target_library_id, state, updated_at_ms DESC, id);
