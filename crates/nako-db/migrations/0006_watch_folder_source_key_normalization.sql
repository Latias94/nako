CREATE TEMP TABLE watch_folder_source_key_normalization AS
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

UPDATE acquisition_intake_candidates
SET
    managed_import_artifact_id = COALESCE(
        managed_import_artifact_id,
        (
            SELECT candidate.managed_import_artifact_id
            FROM acquisition_intake_candidates AS candidate
            JOIN watch_folder_source_key_normalization AS normalized
              ON normalized.target_library_id = candidate.target_library_id
             AND normalized.source_kind = candidate.source_kind
             AND normalized.source_kind_key = candidate.source_kind_key
             AND normalized.source_uri = candidate.source_uri
            WHERE normalized.winner_id = acquisition_intake_candidates.id
              AND candidate.managed_import_artifact_id IS NOT NULL
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
            LIMIT 1
        )
    ),
    display_name = COALESCE(
        display_name,
        (
            SELECT candidate.display_name
            FROM acquisition_intake_candidates AS candidate
            JOIN watch_folder_source_key_normalization AS normalized
              ON normalized.target_library_id = candidate.target_library_id
             AND normalized.source_kind = candidate.source_kind
             AND normalized.source_kind_key = candidate.source_kind_key
             AND normalized.source_uri = candidate.source_uri
            WHERE normalized.winner_id = acquisition_intake_candidates.id
              AND candidate.display_name IS NOT NULL
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
            LIMIT 1
        )
    ),
    intended_locator = COALESCE(
        intended_locator,
        (
            SELECT candidate.intended_locator
            FROM acquisition_intake_candidates AS candidate
            JOIN watch_folder_source_key_normalization AS normalized
              ON normalized.target_library_id = candidate.target_library_id
             AND normalized.source_kind = candidate.source_kind
             AND normalized.source_kind_key = candidate.source_kind_key
             AND normalized.source_uri = candidate.source_uri
            WHERE normalized.winner_id = acquisition_intake_candidates.id
              AND candidate.intended_locator IS NOT NULL
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
            LIMIT 1
        )
    ),
    size_bytes = COALESCE(
        size_bytes,
        (
            SELECT candidate.size_bytes
            FROM acquisition_intake_candidates AS candidate
            JOIN watch_folder_source_key_normalization AS normalized
              ON normalized.target_library_id = candidate.target_library_id
             AND normalized.source_kind = candidate.source_kind
             AND normalized.source_kind_key = candidate.source_kind_key
             AND normalized.source_uri = candidate.source_uri
            WHERE normalized.winner_id = acquisition_intake_candidates.id
              AND candidate.size_bytes IS NOT NULL
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
            LIMIT 1
        )
    ),
    fingerprint = COALESCE(
        fingerprint,
        (
            SELECT candidate.fingerprint
            FROM acquisition_intake_candidates AS candidate
            JOIN watch_folder_source_key_normalization AS normalized
              ON normalized.target_library_id = candidate.target_library_id
             AND normalized.source_kind = candidate.source_kind
             AND normalized.source_kind_key = candidate.source_kind_key
             AND normalized.source_uri = candidate.source_uri
            WHERE normalized.winner_id = acquisition_intake_candidates.id
              AND candidate.fingerprint IS NOT NULL
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
            LIMIT 1
        )
    ),
    diagnostics_json = COALESCE(
        diagnostics_json,
        (
            SELECT candidate.diagnostics_json
            FROM acquisition_intake_candidates AS candidate
            JOIN watch_folder_source_key_normalization AS normalized
              ON normalized.target_library_id = candidate.target_library_id
             AND normalized.source_kind = candidate.source_kind
             AND normalized.source_kind_key = candidate.source_kind_key
             AND normalized.source_uri = candidate.source_uri
            WHERE normalized.winner_id = acquisition_intake_candidates.id
              AND candidate.diagnostics_json IS NOT NULL
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
            LIMIT 1
        )
    ),
    first_seen_at_ms = (
        SELECT MIN(candidate.first_seen_at_ms)
        FROM acquisition_intake_candidates AS candidate
        JOIN watch_folder_source_key_normalization AS normalized
          ON normalized.target_library_id = candidate.target_library_id
         AND normalized.source_kind = candidate.source_kind
         AND normalized.source_kind_key = candidate.source_kind_key
         AND normalized.source_uri = candidate.source_uri
        WHERE normalized.winner_id = acquisition_intake_candidates.id
    ),
    last_seen_at_ms = (
        SELECT MAX(candidate.last_seen_at_ms)
        FROM acquisition_intake_candidates AS candidate
        JOIN watch_folder_source_key_normalization AS normalized
          ON normalized.target_library_id = candidate.target_library_id
         AND normalized.source_kind = candidate.source_kind
         AND normalized.source_kind_key = candidate.source_kind_key
         AND normalized.source_uri = candidate.source_uri
        WHERE normalized.winner_id = acquisition_intake_candidates.id
    ),
    updated_at_ms = (
        SELECT MAX(candidate.updated_at_ms)
        FROM acquisition_intake_candidates AS candidate
        JOIN watch_folder_source_key_normalization AS normalized
          ON normalized.target_library_id = candidate.target_library_id
         AND normalized.source_kind = candidate.source_kind
         AND normalized.source_kind_key = candidate.source_kind_key
         AND normalized.source_uri = candidate.source_uri
        WHERE normalized.winner_id = acquisition_intake_candidates.id
    ),
    updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
WHERE id IN (
    SELECT winner_id
    FROM watch_folder_source_key_normalization
);

DELETE FROM acquisition_intake_candidates
WHERE id IN (
    SELECT candidate.id
    FROM acquisition_intake_candidates AS candidate
    JOIN watch_folder_source_key_normalization AS normalized
      ON normalized.target_library_id = candidate.target_library_id
     AND normalized.source_kind = candidate.source_kind
     AND normalized.source_kind_key = candidate.source_kind_key
     AND normalized.source_uri = candidate.source_uri
    WHERE candidate.id <> normalized.winner_id
);

UPDATE acquisition_intake_candidates
SET
    source_key = (
        SELECT normalized.canonical_source_key
        FROM watch_folder_source_key_normalization AS normalized
        WHERE normalized.winner_id = acquisition_intake_candidates.id
    ),
    updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
WHERE id IN (
    SELECT winner_id
    FROM watch_folder_source_key_normalization
);

DROP TABLE watch_folder_source_key_normalization;
