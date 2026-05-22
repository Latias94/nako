CREATE TABLE managed_artwork_ingests (
    id TEXT PRIMARY KEY NOT NULL,
    candidate_id TEXT NOT NULL UNIQUE REFERENCES addon_artwork_candidates(id) ON DELETE CASCADE,
    job_id TEXT NOT NULL UNIQUE REFERENCES jobs(id) ON DELETE CASCADE,
    library_id TEXT NOT NULL REFERENCES libraries(id) ON DELETE CASCADE,
    item_id TEXT NOT NULL REFERENCES media_items(id) ON DELETE CASCADE,
    kind TEXT NOT NULL,
    kind_key TEXT NOT NULL DEFAULT '',
    status TEXT NOT NULL CHECK(status IN ('queued', 'fetching', 'validating', 'stored', 'failed')),
    artifact_id TEXT,
    failure_code TEXT CHECK(failure_code IS NULL OR length(failure_code) <= 128),
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE INDEX managed_artwork_ingests_item_idx
    ON managed_artwork_ingests(item_id, status, kind, kind_key);

CREATE INDEX managed_artwork_ingests_job_idx
    ON managed_artwork_ingests(job_id);

CREATE TABLE managed_artwork_artifacts (
    id TEXT PRIMARY KEY NOT NULL,
    ingest_id TEXT NOT NULL UNIQUE REFERENCES managed_artwork_ingests(id) ON DELETE CASCADE,
    library_id TEXT NOT NULL REFERENCES libraries(id) ON DELETE CASCADE,
    item_id TEXT NOT NULL REFERENCES media_items(id) ON DELETE CASCADE,
    kind TEXT NOT NULL,
    kind_key TEXT NOT NULL DEFAULT '',
    storage_uri TEXT NOT NULL,
    content_hash TEXT,
    width INTEGER CHECK(width IS NULL OR (width BETWEEN 1 AND 20000)),
    height INTEGER CHECK(height IS NULL OR (height BETWEEN 1 AND 20000)),
    byte_len INTEGER CHECK(byte_len IS NULL OR byte_len >= 0),
    media_type TEXT CHECK(media_type IS NULL OR length(media_type) <= 128),
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE INDEX managed_artwork_artifacts_item_idx
    ON managed_artwork_artifacts(item_id, kind, kind_key);
