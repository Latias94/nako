ALTER TABLE jobs ADD COLUMN worker_id TEXT;
ALTER TABLE jobs ADD COLUMN run_token TEXT;
ALTER TABLE jobs ADD COLUMN heartbeat_at TEXT;
ALTER TABLE jobs ADD COLUMN lease_expires_at TEXT;
ALTER TABLE jobs ADD COLUMN cancel_requested_at TEXT;
ALTER TABLE jobs ADD COLUMN cancel_reason TEXT;

CREATE INDEX jobs_lease_claim_idx
    ON jobs(status, kind, resource_class, queued_at, id);

CREATE INDEX jobs_lease_expiry_idx
    ON jobs(status, lease_expires_at);
