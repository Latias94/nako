ALTER TABLE staging_manifest_records
    ADD COLUMN attribution_kind text NOT NULL DEFAULT 'unknown';

ALTER TABLE staging_manifest_records
    ADD COLUMN attribution_library_id uuid;

ALTER TABLE staging_manifest_records
    ADD CONSTRAINT staging_manifest_attribution_shape_chk
    CHECK (
        (attribution_kind = 'attributed' AND attribution_library_id IS NOT NULL)
        OR (attribution_kind IN ('ambiguous', 'unknown') AND attribution_library_id IS NULL)
    );

CREATE INDEX IF NOT EXISTS idx_staging_manifest_attribution
    ON staging_manifest_records(attribution_kind, attribution_library_id);
