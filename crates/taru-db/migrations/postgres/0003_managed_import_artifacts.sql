CREATE TABLE IF NOT EXISTS managed_import_artifacts (
    id uuid PRIMARY KEY NOT NULL,
    target_library_id uuid NOT NULL REFERENCES libraries(id) ON DELETE CASCADE,
    source_kind text NOT NULL,
    source_kind_key text NOT NULL DEFAULT '',
    source_uri text NOT NULL,
    staging_manifest_id uuid REFERENCES staging_manifest_records(id) ON DELETE SET NULL,
    artifact_uri text,
    original_file_name text,
    intended_locator text,
    size_bytes bigint,
    fingerprint text,
    state text NOT NULL,
    diagnostics_json text,
    created_at_ms bigint NOT NULL,
    updated_at_ms bigint NOT NULL,
    created_at timestamptz NOT NULL DEFAULT statement_timestamp(),
    updated_at timestamptz NOT NULL DEFAULT statement_timestamp(),
    UNIQUE(target_library_id, source_kind, source_kind_key, source_uri),
    CHECK (length(source_uri) > 0),
    CHECK (size_bytes IS NULL OR size_bytes >= 0)
);

CREATE INDEX IF NOT EXISTS managed_import_artifacts_library_state_idx
    ON managed_import_artifacts(target_library_id, state, updated_at_ms DESC, id);

CREATE INDEX IF NOT EXISTS managed_import_artifacts_source_kind_idx
    ON managed_import_artifacts(target_library_id, source_kind, source_kind_key, source_uri);

CREATE INDEX IF NOT EXISTS managed_import_artifacts_staging_manifest_idx
    ON managed_import_artifacts(staging_manifest_id);

CREATE INDEX IF NOT EXISTS managed_import_artifacts_fingerprint_idx
    ON managed_import_artifacts(target_library_id, fingerprint)
    WHERE fingerprint IS NOT NULL;
