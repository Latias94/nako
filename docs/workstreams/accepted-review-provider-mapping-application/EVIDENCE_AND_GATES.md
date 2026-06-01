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
- `cargo nextest run -p nako-metadata --no-fail-fast`
- `cargo nextest run -p nako-core --no-fail-fast`
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

## ARPMA-020 Evidence

Red check:

- `cargo nextest run -p nako-metadata candidate_review_application --no-fail-fast`
  failed before implementation because
  `build_candidate_review_application_plan`,
  `MetadataCandidateReviewApplicationAction`, and
  `MetadataCandidateReviewApplicationReason` did not exist.

Implemented behavior:

- Added `MetadataCandidateReviewApplicationPlan`,
  `MetadataCandidateReviewApplicationAction`, and
  `MetadataCandidateReviewApplicationReason` in `nako-core`.
- Added read-only `build_candidate_review_application_plan` in
  `nako-metadata`.
- Plans expose source conversion from `MetadataCandidateSource` to
  `MetadataSource`, and unsupported sources remain explicit skip reasons.
- Plans inspect existing Provider Mapping state without upserting Provider
  Subjects or Provider Mappings.
- Existing accepted mappings become `Noop`, existing rejected mappings become
  `Skip`, existing candidate mappings stay `Apply`, and ready new mappings are
  `Apply`.

Green checks:

- `cargo nextest run -p nako-metadata candidate_review_application --no-fail-fast`
  passed: 3 tests run, 3 passed.
- `cargo nextest run -p nako-metadata --no-fail-fast` passed: 48 tests run,
  48 passed.
- `cargo nextest run -p nako-core --no-fail-fast` passed: 35 tests run,
  35 passed.
- `cargo fmt --all -- --check` passed.
- `python -m json.tool docs/workstreams/accepted-review-provider-mapping-application/WORKSTREAM.json`
  passed.
- JSONL validation for `TASKS.jsonl`, `CAMPAIGNS.jsonl`, and `CONTEXT.jsonl`
  passed.
- `git diff --check` passed.

## ARPMA-030 Evidence

Red check:

- `cargo nextest run -p nako-metadata candidate_review_application --no-fail-fast`
  failed before implementation because
  `MetadataCandidateReviewApplicationRequest` and
  `MetadataCandidateReviewApplicationService` did not exist.

Implemented behavior:

- Added `MetadataCandidateReviewApplicationService` in `nako-metadata`.
- Application loads a durable review, enforces item and freshness guards, then
  uses the read-only application plan before writing anything.
- Wrong-item and stale expected-update requests return conflicts without
  Provider Mapping mutation.
- Accepted reviews write or promote only the root Provider Subject and root
  Provider Mapping through `ProviderMappingRepository`.
- Replays of an already accepted root mapping return `Noop` without duplicate
  Provider Mapping rows.
- Existing rejected Provider Mappings return a conflict and are not overwritten.
- Related review nodes and relationships remain preview evidence and are not
  persisted by this service.

Green checks:

- `cargo nextest run -p nako-metadata candidate_review_application --no-fail-fast`
  passed: 6 tests run, 6 passed.
- `cargo nextest run -p nako-metadata --no-fail-fast` passed: 51 tests run,
  51 passed.
- `cargo nextest run -p nako-db candidate_review provider_mapping --no-fail-fast`
  passed: 3 tests run, 3 passed.
- `cargo fmt --all -- --check` passed.
- `python -m json.tool docs/workstreams/accepted-review-provider-mapping-application/WORKSTREAM.json`
  passed.
- JSONL validation for `TASKS.jsonl`, `CAMPAIGNS.jsonl`, and `CONTEXT.jsonl`
  passed.
- `git diff --check` passed with Git CRLF normalization warnings only.
