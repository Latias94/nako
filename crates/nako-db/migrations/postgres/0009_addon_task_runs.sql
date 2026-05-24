CREATE TABLE IF NOT EXISTS addon_task_runs (
    job_id uuid PRIMARY KEY NOT NULL REFERENCES jobs(id) ON DELETE CASCADE,
    addon_id uuid NOT NULL REFERENCES addon_registrations(id) ON DELETE CASCADE,
    manifest_id text NOT NULL,
    manifest_version text NOT NULL,
    manifest_fingerprint text NOT NULL,
    declaration_id text NOT NULL,
    declaration_name text NOT NULL,
    declaration_path text NOT NULL,
    idempotency_key text NOT NULL,
    request_fingerprint text NOT NULL,
    attempt bigint NOT NULL,
    max_attempts bigint,
    retry_of_job_id uuid REFERENCES jobs(id) ON DELETE SET NULL,
    input_json text NOT NULL,
    progress_json text,
    result_json text,
    safe_error_code text,
    created_at timestamptz NOT NULL DEFAULT statement_timestamp(),
    updated_at timestamptz NOT NULL DEFAULT statement_timestamp(),
    UNIQUE(addon_id, idempotency_key)
);

CREATE INDEX IF NOT EXISTS addon_task_runs_addon_declaration_idx
    ON addon_task_runs(addon_id, declaration_id, created_at, job_id);

CREATE INDEX IF NOT EXISTS addon_task_runs_retry_idx
    ON addon_task_runs(retry_of_job_id);
