ALTER TABLE jobs
    ADD COLUMN priority bigint NOT NULL DEFAULT 50;

DROP INDEX IF EXISTS jobs_lease_claim_idx;

CREATE INDEX IF NOT EXISTS jobs_lease_claim_idx
    ON jobs(status, kind, resource_class, library_id, source_id, priority, next_attempt_at, queued_at, id);

CREATE INDEX IF NOT EXISTS jobs_priority_idx
    ON jobs(priority, queued_at, id);
