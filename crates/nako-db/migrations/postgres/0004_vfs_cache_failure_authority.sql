ALTER TABLE vfs_cache_failures
    ADD COLUMN IF NOT EXISTS library_id uuid;

ALTER TABLE vfs_cache_failures
    ADD COLUMN IF NOT EXISTS backend_key text;

CREATE INDEX IF NOT EXISTS vfs_cache_failures_authority_idx
    ON vfs_cache_failures(library_id, backend_key, failed_at_ms);
