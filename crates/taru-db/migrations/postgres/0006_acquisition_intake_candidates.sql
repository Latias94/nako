CREATE TABLE IF NOT EXISTS acquisition_intake_candidates (
    id uuid PRIMARY KEY NOT NULL,
    target_library_id uuid NOT NULL REFERENCES libraries(id) ON DELETE CASCADE,
    source_kind text NOT NULL,
    source_kind_key text NOT NULL DEFAULT '',
    source_key text NOT NULL,
    source_uri text NOT NULL,
    display_name text,
    intended_locator text,
    size_bytes bigint,
    fingerprint text,
    managed_import_artifact_id uuid REFERENCES managed_import_artifacts(id) ON DELETE SET NULL,
    state text NOT NULL,
    diagnostics_json text,
    first_seen_at_ms bigint NOT NULL,
    last_seen_at_ms bigint NOT NULL,
    created_at_ms bigint NOT NULL,
    updated_at_ms bigint NOT NULL,
    created_at timestamptz NOT NULL DEFAULT statement_timestamp(),
    updated_at timestamptz NOT NULL DEFAULT statement_timestamp(),
    UNIQUE(target_library_id, source_kind, source_kind_key, source_key),
    CHECK (length(source_key) > 0),
    CHECK (length(source_uri) > 0),
    CHECK (size_bytes IS NULL OR size_bytes >= 0)
);

CREATE INDEX IF NOT EXISTS acquisition_intake_candidates_library_state_idx
    ON acquisition_intake_candidates(target_library_id, state, updated_at_ms DESC, id);

CREATE INDEX IF NOT EXISTS acquisition_intake_candidates_source_kind_idx
    ON acquisition_intake_candidates(target_library_id, source_kind, source_kind_key, source_key);

CREATE INDEX IF NOT EXISTS acquisition_intake_candidates_managed_import_idx
    ON acquisition_intake_candidates(managed_import_artifact_id)
    WHERE managed_import_artifact_id IS NOT NULL;

CREATE INDEX IF NOT EXISTS acquisition_intake_candidates_fingerprint_idx
    ON acquisition_intake_candidates(target_library_id, fingerprint)
    WHERE fingerprint IS NOT NULL;
