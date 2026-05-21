CREATE TABLE acquisition_intake_candidates (
    id TEXT PRIMARY KEY NOT NULL,
    target_library_id TEXT NOT NULL REFERENCES libraries(id) ON DELETE CASCADE,
    source_kind TEXT NOT NULL,
    source_kind_key TEXT NOT NULL DEFAULT '',
    source_key TEXT NOT NULL,
    source_uri TEXT NOT NULL,
    display_name TEXT,
    intended_locator TEXT,
    size_bytes INTEGER,
    fingerprint TEXT,
    managed_import_artifact_id TEXT REFERENCES managed_import_artifacts(id) ON DELETE SET NULL,
    state TEXT NOT NULL,
    diagnostics_json TEXT,
    first_seen_at_ms INTEGER NOT NULL,
    last_seen_at_ms INTEGER NOT NULL,
    created_at_ms INTEGER NOT NULL,
    updated_at_ms INTEGER NOT NULL,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    UNIQUE(target_library_id, source_kind, source_kind_key, source_key),
    CHECK (length(source_key) > 0),
    CHECK (length(source_uri) > 0),
    CHECK (size_bytes IS NULL OR size_bytes >= 0)
);

CREATE INDEX acquisition_intake_candidates_library_state_idx
    ON acquisition_intake_candidates(target_library_id, state, updated_at_ms DESC, id);

CREATE INDEX acquisition_intake_candidates_source_kind_idx
    ON acquisition_intake_candidates(target_library_id, source_kind, source_kind_key, source_key);

CREATE INDEX acquisition_intake_candidates_managed_import_idx
    ON acquisition_intake_candidates(managed_import_artifact_id)
    WHERE managed_import_artifact_id IS NOT NULL;

CREATE INDEX acquisition_intake_candidates_fingerprint_idx
    ON acquisition_intake_candidates(target_library_id, fingerprint)
    WHERE fingerprint IS NOT NULL;
