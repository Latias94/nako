# Accepted Review Provider Mapping Application - Evidence And Gates

Status: Active
Last updated: 2026-06-02

## Lane Opening Gates

```bash
python -m json.tool docs/workstreams/accepted-review-provider-mapping-application/WORKSTREAM.json
```

```bash
python - <<'PY'
import json
from pathlib import Path
for rel in [
    "docs/workstreams/accepted-review-provider-mapping-application/TASKS.jsonl",
    "docs/workstreams/accepted-review-provider-mapping-application/CAMPAIGNS.jsonl",
    "docs/workstreams/accepted-review-provider-mapping-application/CONTEXT.jsonl",
]:
    for line in Path(rel).read_text(encoding="utf-8").splitlines():
        if line.strip():
            json.loads(line)
print("jsonl ok")
PY
```

```bash
git diff --check
```

## Expected Gates

- `cargo nextest run -p nako-metadata candidate_review_application --no-fail-fast`
- `cargo nextest run -p nako-db candidate_review provider_mapping --no-fail-fast`
- `cargo fmt --all -- --check`
- `git diff --check`

## Evidence Anchors

- `docs/workstreams/metadata-candidate-durable-review/CLOSEOUT.md`
- `docs/workstreams/generated-artifact-provider-mapping-breadth/CLOSEOUT.md`
- `docs/architecture/LIBRARY_PIPELINE.md`
- `crates/nako-metadata/src/candidate_review.rs`
- `crates/nako-metadata/src/confirmation.rs`
- `crates/nako-core/src/media/candidate.rs`
- `crates/nako-core/src/repository/metadata.rs`

## Opening Recon

Local recon on 2026-06-02:

- `MetadataCandidateReviewDecisionService` accepts/rejects review status only
  and depends only on `MetadataCandidateReviewRepository`.
- `MetadataCandidateReviewPlan` stores root and related Provider Subject
  summaries, but only the root subject should be eligible for this lane.
- `MetadataCandidateSource` includes `Automation` and `Other`, which do not
  have a safe direct `MetadataSource` conversion today.
- `HierarchyConfirmationService` and Generated Artifact Provider Mapping apply
  already prove accepted Provider Mapping writes can be idempotent through
  `ProviderMappingRepository`.
- Generated Artifact apply outcome tables must not become candidate review
  application state.

## ARPMA-010 Evidence

Implemented behavior:

- opened this workstream as the accepted-review Provider Mapping application
  follow-on from MCDR closeout;
- routed the active queue to `ARPMA-020`;
- kept Admin/Web provider depth governance split.

Green checks:

- `python -m json.tool docs/workstreams/accepted-review-provider-mapping-application/WORKSTREAM.json`
- JSONL validation for `TASKS.jsonl`, `CAMPAIGNS.jsonl`, and `CONTEXT.jsonl`
- `git diff --check`
