CREATE TABLE addon_task_runs (
    job_id TEXT PRIMARY KEY NOT NULL REFERENCES jobs(id) ON DELETE CASCADE,
    addon_id TEXT NOT NULL REFERENCES addon_registrations(id) ON DELETE CASCADE,
    manifest_id TEXT NOT NULL,
    manifest_version TEXT NOT NULL,
    manifest_fingerprint TEXT NOT NULL,
    declaration_id TEXT NOT NULL,
    declaration_name TEXT NOT NULL,
    declaration_path TEXT NOT NULL,
    idempotency_key TEXT NOT NULL,
    request_fingerprint TEXT NOT NULL,
    attempt INTEGER NOT NULL,
    max_attempts INTEGER,
    retry_of_job_id TEXT REFERENCES jobs(id) ON DELETE SET NULL,
    input_json TEXT NOT NULL,
    progress_json TEXT,
    result_json TEXT,
    safe_error_code TEXT,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    UNIQUE(addon_id, idempotency_key)
);

CREATE INDEX addon_task_runs_addon_declaration_idx
    ON addon_task_runs(addon_id, declaration_id, created_at, job_id);

CREATE INDEX addon_task_runs_retry_idx
    ON addon_task_runs(retry_of_job_id);
