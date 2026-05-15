CREATE TABLE IF NOT EXISTS staging_manifest_records (
    id TEXT PRIMARY KEY NOT NULL,
    source_uri TEXT NOT NULL,
    source_scheme TEXT NOT NULL,
    purpose TEXT NOT NULL,
    local_path TEXT NOT NULL UNIQUE,
    size_bytes INTEGER,
    etag TEXT,
    fingerprint TEXT,
    state TEXT NOT NULL,
    created_at_ms INTEGER NOT NULL,
    updated_at_ms INTEGER NOT NULL,
    last_accessed_at_ms INTEGER NOT NULL,
    expires_at_ms INTEGER,
    active_leases INTEGER NOT NULL DEFAULT 0,
    validation_error TEXT
);

CREATE INDEX IF NOT EXISTS idx_staging_manifest_source_purpose
ON staging_manifest_records (source_uri, purpose);

CREATE INDEX IF NOT EXISTS idx_staging_manifest_cleanup
ON staging_manifest_records (state, active_leases, expires_at_ms, last_accessed_at_ms);
