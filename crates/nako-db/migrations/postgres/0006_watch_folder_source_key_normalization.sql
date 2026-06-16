CREATE TEMP TABLE watch_folder_source_key_normalization ON COMMIT DROP AS
WITH affected_groups AS (
    SELECT
        target_library_id,
        source_kind,
        source_kind_key,
        source_uri
    FROM acquisition_intake_candidates
    WHERE source_kind = 'watch_folder'
      AND source_kind_key = ''
      AND source_key <> ('watch_folder:' || source_uri)
    GROUP BY target_library_id, source_kind, source_kind_key, source_uri
),
ranked_candidates AS (
    SELECT
        candidate.id,
        candidate.target_library_id,
        candidate.source_kind,
        candidate.source_kind_key,
        candidate.source_uri,
        'watch_folder:' || candidate.source_uri AS canonical_source_key,
        row_number() OVER (
            PARTITION BY
                candidate.target_library_id,
                candidate.source_kind,
                candidate.source_kind_key,
                candidate.source_uri
            ORDER BY
                CASE
                    WHEN candidate.state = 'accepted'
                         AND candidate.managed_import_artifact_id IS NOT NULL THEN 0
                    WHEN candidate.state = 'accepted' THEN 1
                    WHEN candidate.managed_import_artifact_id IS NOT NULL THEN 2
                    ELSE 3
                END ASC,
                candidate.last_seen_at_ms DESC,
                candidate.updated_at_ms DESC,
                candidate.created_at_ms DESC,
                candidate.id ASC
        ) AS candidate_rank
    FROM acquisition_intake_candidates AS candidate
    JOIN affected_groups AS affected
      ON affected.target_library_id = candidate.target_library_id
     AND affected.source_kind = candidate.source_kind
     AND affected.source_kind_key = candidate.source_kind_key
     AND affected.source_uri = candidate.source_uri
)
SELECT
    id AS winner_id,
    target_library_id,
    source_kind,
    source_kind_key,
    source_uri,
    canonical_source_key
FROM ranked_candidates
WHERE candidate_rank = 1;

WITH merged AS (
    SELECT
        normalized.winner_id,
        MIN(candidate.first_seen_at_ms) AS first_seen_at_ms,
        MAX(candidate.last_seen_at_ms) AS last_seen_at_ms,
        MAX(candidate.updated_at_ms) AS updated_at_ms,
        (
            array_agg(
                candidate.managed_import_artifact_id
                ORDER BY
                    CASE
                        WHEN candidate.state = 'accepted'
                             AND candidate.managed_import_artifact_id IS NOT NULL THEN 0
                        WHEN candidate.state = 'accepted' THEN 1
                        WHEN candidate.managed_import_artifact_id IS NOT NULL THEN 2
                        ELSE 3
                    END ASC,
                    candidate.last_seen_at_ms DESC,
                    candidate.updated_at_ms DESC,
                    candidate.created_at_ms DESC,
                    candidate.id ASC
            ) FILTER (WHERE candidate.managed_import_artifact_id IS NOT NULL)
        )[1] AS managed_import_artifact_id,
        (
            array_agg(
                candidate.display_name
                ORDER BY
                    CASE
                        WHEN candidate.state = 'accepted'
                             AND candidate.managed_import_artifact_id IS NOT NULL THEN 0
                        WHEN candidate.state = 'accepted' THEN 1
                        WHEN candidate.managed_import_artifact_id IS NOT NULL THEN 2
                        ELSE 3
                    END ASC,
                    candidate.last_seen_at_ms DESC,
                    candidate.updated_at_ms DESC,
                    candidate.created_at_ms DESC,
                    candidate.id ASC
            ) FILTER (WHERE candidate.display_name IS NOT NULL)
        )[1] AS display_name,
        (
            array_agg(
                candidate.intended_locator
                ORDER BY
                    CASE
                        WHEN candidate.state = 'accepted'
                             AND candidate.managed_import_artifact_id IS NOT NULL THEN 0
                        WHEN candidate.state = 'accepted' THEN 1
                        WHEN candidate.managed_import_artifact_id IS NOT NULL THEN 2
                        ELSE 3
                    END ASC,
                    candidate.last_seen_at_ms DESC,
                    candidate.updated_at_ms DESC,
                    candidate.created_at_ms DESC,
                    candidate.id ASC
            ) FILTER (WHERE candidate.intended_locator IS NOT NULL)
        )[1] AS intended_locator,
        (
            array_agg(
                candidate.size_bytes
                ORDER BY
                    CASE
                        WHEN candidate.state = 'accepted'
                             AND candidate.managed_import_artifact_id IS NOT NULL THEN 0
                        WHEN candidate.state = 'accepted' THEN 1
                        WHEN candidate.managed_import_artifact_id IS NOT NULL THEN 2
                        ELSE 3
                    END ASC,
                    candidate.last_seen_at_ms DESC,
                    candidate.updated_at_ms DESC,
                    candidate.created_at_ms DESC,
                    candidate.id ASC
            ) FILTER (WHERE candidate.size_bytes IS NOT NULL)
        )[1] AS size_bytes,
        (
            array_agg(
                candidate.fingerprint
                ORDER BY
                    CASE
                        WHEN candidate.state = 'accepted'
                             AND candidate.managed_import_artifact_id IS NOT NULL THEN 0
                        WHEN candidate.state = 'accepted' THEN 1
                        WHEN candidate.managed_import_artifact_id IS NOT NULL THEN 2
                        ELSE 3
                    END ASC,
                    candidate.last_seen_at_ms DESC,
                    candidate.updated_at_ms DESC,
                    candidate.created_at_ms DESC,
                    candidate.id ASC
            ) FILTER (WHERE candidate.fingerprint IS NOT NULL)
        )[1] AS fingerprint,
        (
            array_agg(
                candidate.diagnostics_json
                ORDER BY
                    CASE
                        WHEN candidate.state = 'accepted'
                             AND candidate.managed_import_artifact_id IS NOT NULL THEN 0
                        WHEN candidate.state = 'accepted' THEN 1
                        WHEN candidate.managed_import_artifact_id IS NOT NULL THEN 2
                        ELSE 3
                    END ASC,
                    candidate.last_seen_at_ms DESC,
                    candidate.updated_at_ms DESC,
                    candidate.created_at_ms DESC,
                    candidate.id ASC
            ) FILTER (WHERE candidate.diagnostics_json IS NOT NULL)
        )[1] AS diagnostics_json
    FROM watch_folder_source_key_normalization AS normalized
    JOIN acquisition_intake_candidates AS candidate
      ON normalized.target_library_id = candidate.target_library_id
     AND normalized.source_kind = candidate.source_kind
     AND normalized.source_kind_key = candidate.source_kind_key
     AND normalized.source_uri = candidate.source_uri
    GROUP BY normalized.winner_id
)
UPDATE acquisition_intake_candidates AS candidate
SET
    managed_import_artifact_id = COALESCE(
        candidate.managed_import_artifact_id,
        merged.managed_import_artifact_id
    ),
    display_name = COALESCE(candidate.display_name, merged.display_name),
    intended_locator = COALESCE(candidate.intended_locator, merged.intended_locator),
    size_bytes = COALESCE(candidate.size_bytes, merged.size_bytes),
    fingerprint = COALESCE(candidate.fingerprint, merged.fingerprint),
    diagnostics_json = COALESCE(candidate.diagnostics_json, merged.diagnostics_json),
    first_seen_at_ms = merged.first_seen_at_ms,
    last_seen_at_ms = merged.last_seen_at_ms,
    updated_at_ms = merged.updated_at_ms,
    updated_at = statement_timestamp()
FROM merged
WHERE candidate.id = merged.winner_id;

DELETE FROM acquisition_intake_candidates AS candidate
USING watch_folder_source_key_normalization AS normalized
WHERE normalized.target_library_id = candidate.target_library_id
  AND normalized.source_kind = candidate.source_kind
  AND normalized.source_kind_key = candidate.source_kind_key
  AND normalized.source_uri = candidate.source_uri
  AND candidate.id <> normalized.winner_id;

UPDATE acquisition_intake_candidates AS candidate
SET
    source_key = normalized.canonical_source_key,
    updated_at = statement_timestamp()
FROM watch_folder_source_key_normalization AS normalized
WHERE candidate.id = normalized.winner_id;
