CREATE TABLE metadata_provider_attempts (
    id TEXT PRIMARY KEY NOT NULL,
    job_id TEXT NOT NULL REFERENCES jobs(id) ON DELETE CASCADE,
    item_id TEXT NOT NULL REFERENCES media_items(id) ON DELETE CASCADE,
    provider TEXT NOT NULL,
    provider_key TEXT,
    status TEXT NOT NULL,
    matched_by TEXT,
    started_at TEXT NOT NULL,
    finished_at TEXT NOT NULL,
    error_class TEXT,
    message TEXT,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE INDEX metadata_provider_attempts_job_id_idx
    ON metadata_provider_attempts(job_id, started_at);

CREATE INDEX metadata_provider_attempts_item_id_idx
    ON metadata_provider_attempts(item_id, started_at);
