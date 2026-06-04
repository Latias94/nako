ALTER TABLE vfs_cache_failures
    ADD COLUMN library_id TEXT;

ALTER TABLE vfs_cache_failures
    ADD COLUMN backend_key TEXT;

CREATE INDEX vfs_cache_failures_authority_idx
    ON vfs_cache_failures(library_id, backend_key, failed_at_ms);
