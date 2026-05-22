ALTER TABLE managed_artwork_artifacts
    ADD COLUMN deleted_at TEXT;

CREATE INDEX managed_artwork_artifacts_deleted_idx
    ON managed_artwork_artifacts(deleted_at, created_at, id);
