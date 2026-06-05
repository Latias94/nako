UPDATE source_duplicate_relationships
SET
    source_id = duplicate_source_id,
    duplicate_source_id = source_id
WHERE source_id > duplicate_source_id;

WITH latest_payload AS (
    SELECT DISTINCT ON (source_id, duplicate_source_id)
        source_id,
        duplicate_source_id,
        evidence_kind,
        evidence_kind_key,
        evidence_value,
        status,
        confidence_milli,
        updated_at
    FROM source_duplicate_relationships
    ORDER BY source_id, duplicate_source_id, updated_at DESC, created_at DESC, id DESC
),
keep_rows AS (
    SELECT DISTINCT ON (source_id, duplicate_source_id)
        id,
        source_id,
        duplicate_source_id
    FROM source_duplicate_relationships
    ORDER BY source_id, duplicate_source_id, created_at ASC, id ASC
)
UPDATE source_duplicate_relationships AS relationship
SET
    evidence_kind = latest_payload.evidence_kind,
    evidence_kind_key = latest_payload.evidence_kind_key,
    evidence_value = latest_payload.evidence_value,
    status = latest_payload.status,
    confidence_milli = latest_payload.confidence_milli,
    updated_at = latest_payload.updated_at
FROM keep_rows, latest_payload
WHERE relationship.id = keep_rows.id
  AND keep_rows.source_id = latest_payload.source_id
  AND keep_rows.duplicate_source_id = latest_payload.duplicate_source_id;

DELETE FROM source_duplicate_relationships AS relationship
USING (
    SELECT
        id,
        row_number() OVER (
            PARTITION BY source_id, duplicate_source_id
            ORDER BY created_at ASC, id ASC
        ) AS duplicate_rank
    FROM source_duplicate_relationships
) AS ranked
WHERE relationship.id = ranked.id
  AND ranked.duplicate_rank > 1;

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1
        FROM pg_constraint
        WHERE conname = 'source_duplicate_relationships_canonical_pair_check'
          AND conrelid = 'source_duplicate_relationships'::regclass
    ) THEN
        ALTER TABLE source_duplicate_relationships
            ADD CONSTRAINT source_duplicate_relationships_canonical_pair_check
            CHECK (source_id < duplicate_source_id);
    END IF;
END $$;

CREATE UNIQUE INDEX IF NOT EXISTS source_duplicate_relationships_pair_idx
    ON source_duplicate_relationships(source_id, duplicate_source_id);
