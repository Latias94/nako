CREATE TABLE jobs (
    id TEXT PRIMARY KEY NOT NULL,
    kind TEXT NOT NULL,
    status TEXT NOT NULL,
    resource_class TEXT NOT NULL,
    library_id TEXT REFERENCES libraries(id) ON DELETE SET NULL,
    source_id TEXT REFERENCES media_sources(id) ON DELETE SET NULL,
    summary_json TEXT,
    error TEXT,
    queued_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    started_at TEXT,
    completed_at TEXT,
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE INDEX jobs_status_idx ON jobs(status);
CREATE INDEX jobs_kind_idx ON jobs(kind);
CREATE INDEX jobs_library_id_idx ON jobs(library_id);
CREATE INDEX jobs_source_id_idx ON jobs(source_id);
