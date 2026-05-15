CREATE TABLE automation_providers (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,
    base_url TEXT NOT NULL,
    secret_env TEXT,
    capabilities_json TEXT NOT NULL,
    timeout_ms INTEGER NOT NULL,
    max_attempts INTEGER NOT NULL,
    status TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE INDEX automation_providers_status_idx
    ON automation_providers(status);

CREATE TABLE automation_artifacts (
    id TEXT PRIMARY KEY NOT NULL,
    job_id TEXT NOT NULL REFERENCES jobs(id) ON DELETE CASCADE,
    provider_id TEXT NOT NULL REFERENCES automation_providers(id) ON DELETE CASCADE,
    capability TEXT NOT NULL,
    kind TEXT NOT NULL,
    library_id TEXT REFERENCES libraries(id) ON DELETE SET NULL,
    item_id TEXT REFERENCES media_items(id) ON DELETE SET NULL,
    source_id TEXT REFERENCES media_sources(id) ON DELETE SET NULL,
    artifact_json TEXT NOT NULL,
    status TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    accepted_at TEXT
);

CREATE INDEX automation_artifacts_job_idx
    ON automation_artifacts(job_id, created_at);

CREATE INDEX automation_artifacts_item_idx
    ON automation_artifacts(item_id, created_at);

CREATE INDEX automation_artifacts_status_idx
    ON automation_artifacts(status, created_at);
