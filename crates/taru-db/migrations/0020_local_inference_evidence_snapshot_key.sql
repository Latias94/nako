DELETE FROM local_inference_evidence
WHERE rowid NOT IN (
    SELECT MAX(rowid)
    FROM local_inference_evidence
    GROUP BY source_id, evidence_source, evidence_source_key, inference_version
);

CREATE UNIQUE INDEX local_inference_evidence_snapshot_idx
    ON local_inference_evidence(
        source_id,
        evidence_source,
        evidence_source_key,
        inference_version
    );
