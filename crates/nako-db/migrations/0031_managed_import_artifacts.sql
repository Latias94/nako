CREATE TABLE managed_import_artifacts (
    id TEXT PRIMARY KEY NOT NULL,
    target_library_id TEXT NOT NULL REFERENCES libraries(id) ON DELETE CASCADE,
    source_kind TEXT NOT NULL,
    source_kind_key TEXT NOT NULL DEFAULT '',
    source_uri TEXT NOT NULL,
    staging_manifest_id TEXT REFERENCES staging_manifest_records(id) ON DELETE SET NULL,
    artifact_uri TEXT,
    original_file_name TEXT,
    intended_locator TEXT,
    size_bytes INTEGER,
    fingerprint TEXT,
    state TEXT NOT NULL,
    diagnostics_json TEXT,
    created_at_ms INTEGER NOT NULL,
    updated_at_ms INTEGER NOT NULL,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    UNIQUE(target_library_id, source_kind, source_kind_key, source_uri),
    CHECK (length(source_uri) > 0),
    CHECK (size_bytes IS NULL OR size_bytes >= 0)
);

CREATE INDEX managed_import_artifacts_library_state_idx
    ON managed_import_artifacts(target_library_id, state, updated_at_ms DESC, id);

CREATE INDEX managed_import_artifacts_source_kind_idx
    ON managed_import_artifacts(target_library_id, source_kind, source_kind_key, source_uri);

CREATE INDEX managed_import_artifacts_staging_manifest_idx
    ON managed_import_artifacts(staging_manifest_id);

CREATE INDEX managed_import_artifacts_fingerprint_idx
    ON managed_import_artifacts(target_library_id, fingerprint)
    WHERE fingerprint IS NOT NULL;
