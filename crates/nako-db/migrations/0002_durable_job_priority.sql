ALTER TABLE jobs
    ADD COLUMN priority INTEGER NOT NULL DEFAULT 50;

DROP INDEX jobs_lease_claim_idx;

CREATE INDEX jobs_lease_claim_idx
    ON jobs(status, kind, resource_class, priority, next_attempt_at, queued_at, id);

CREATE INDEX jobs_priority_idx
    ON jobs(priority, queued_at, id);
