ALTER TABLE staging_manifest_records
    ADD COLUMN attribution_kind TEXT NOT NULL DEFAULT 'unknown';

ALTER TABLE staging_manifest_records
    ADD COLUMN attribution_library_id TEXT;

CREATE INDEX idx_staging_manifest_attribution
    ON staging_manifest_records(attribution_kind, attribution_library_id);

CREATE TRIGGER staging_manifest_attribution_shape_insert
BEFORE INSERT ON staging_manifest_records
WHEN NOT (
    (NEW.attribution_kind = 'attributed' AND NEW.attribution_library_id IS NOT NULL)
    OR (NEW.attribution_kind IN ('ambiguous', 'unknown') AND NEW.attribution_library_id IS NULL)
)
BEGIN
    SELECT RAISE(ABORT, 'invalid staging attribution shape');
END;

CREATE TRIGGER staging_manifest_attribution_shape_update
BEFORE UPDATE OF attribution_kind, attribution_library_id ON staging_manifest_records
WHEN NOT (
    (NEW.attribution_kind = 'attributed' AND NEW.attribution_library_id IS NOT NULL)
    OR (NEW.attribution_kind IN ('ambiguous', 'unknown') AND NEW.attribution_library_id IS NULL)
)
BEGIN
    SELECT RAISE(ABORT, 'invalid staging attribution shape');
END;
