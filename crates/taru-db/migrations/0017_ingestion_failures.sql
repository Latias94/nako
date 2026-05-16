CREATE TABLE ingestion_failures (
    library_id TEXT NOT NULL REFERENCES libraries(id) ON DELETE CASCADE,
    phase TEXT NOT NULL,
    target_uri TEXT NOT NULL,
    target_kind TEXT NOT NULL,
    job_id TEXT,
    scan_id TEXT,
    source_id TEXT,
    failure_class TEXT NOT NULL,
    status TEXT NOT NULL,
    message TEXT NOT NULL,
    retryable INTEGER NOT NULL,
    attempts INTEGER NOT NULL,
    first_failed_at_ms INTEGER NOT NULL,
    last_failed_at_ms INTEGER NOT NULL,
    resolved_at_ms INTEGER,
    ignored_at_ms INTEGER,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    PRIMARY KEY (library_id, phase, target_uri)
);

CREATE INDEX ingestion_failures_library_status_idx
    ON ingestion_failures(library_id, status, phase, target_uri);

CREATE INDEX ingestion_failures_job_idx
    ON ingestion_failures(job_id);

CREATE INDEX ingestion_failures_scan_idx
    ON ingestion_failures(scan_id);

CREATE INDEX ingestion_failures_source_idx
    ON ingestion_failures(source_id);
