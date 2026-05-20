CREATE TABLE IF NOT EXISTS libraries (
    id uuid PRIMARY KEY NOT NULL,
    name text NOT NULL,
    roots_json jsonb NOT NULL,
    options_json jsonb NOT NULL,
    domain text NOT NULL,
    preset text NOT NULL,
    created_at timestamptz NOT NULL DEFAULT statement_timestamp(),
    updated_at timestamptz NOT NULL DEFAULT statement_timestamp()
);

CREATE TABLE IF NOT EXISTS jobs (
    id uuid PRIMARY KEY NOT NULL,
    kind text NOT NULL,
    status text NOT NULL,
    resource_class text NOT NULL,
    library_id uuid REFERENCES libraries(id) ON DELETE SET NULL,
    source_id uuid,
    input_json text,
    summary_json text,
    error text,
    queued_at timestamptz NOT NULL DEFAULT statement_timestamp(),
    started_at timestamptz,
    completed_at timestamptz,
    updated_at timestamptz NOT NULL DEFAULT statement_timestamp(),
    worker_id uuid,
    run_token uuid,
    heartbeat_at timestamptz,
    lease_expires_at timestamptz,
    cancel_requested_at timestamptz,
    cancel_reason text
);

CREATE INDEX IF NOT EXISTS jobs_status_idx ON jobs(status);
CREATE INDEX IF NOT EXISTS jobs_kind_idx ON jobs(kind);
CREATE INDEX IF NOT EXISTS jobs_library_id_idx ON jobs(library_id);
CREATE INDEX IF NOT EXISTS jobs_source_id_idx ON jobs(source_id);
CREATE INDEX IF NOT EXISTS jobs_lease_claim_idx
    ON jobs(status, kind, resource_class, library_id, source_id, queued_at, id);
CREATE INDEX IF NOT EXISTS jobs_lease_expiry_idx
    ON jobs(status, lease_expires_at);
