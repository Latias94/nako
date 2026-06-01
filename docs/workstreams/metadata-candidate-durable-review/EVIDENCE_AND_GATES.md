# Metadata Candidate Durable Review - Evidence And Gates

Status: Closed
Last updated: 2026-06-02

## Lane Opening Gates

```bash
python -m json.tool docs/workstreams/metadata-candidate-durable-review/WORKSTREAM.json
```

```bash
python - <<'PY'
import json
from pathlib import Path
for rel in [
    "docs/workstreams/metadata-candidate-durable-review/TASKS.jsonl",
    "docs/workstreams/metadata-candidate-durable-review/CAMPAIGNS.jsonl",
    "docs/workstreams/metadata-candidate-durable-review/CONTEXT.jsonl",
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

- `cargo nextest run -p nako-metadata candidate_review metadata_candidate --no-fail-fast`
- `cargo nextest run -p nako-db candidate_review --no-fail-fast`
- `cargo nextest run -p nako-db provider_subjects --no-fail-fast`
- `cargo nextest run -p nako-db baseline_migration --no-fail-fast`
- `cargo nextest run -p nako-db --no-fail-fast`
- `cargo nextest run -p nako-metadata candidate_review_decision --no-fail-fast`
- `cargo nextest run -p nako-metadata --no-fail-fast`
- `cargo fmt --all -- --check`
- `git diff --check`

## Evidence Anchors

- `docs/workstreams/metadata-provider-depth-and-precision/FOLLOW_ONS.md`
- `docs/workstreams/tmdb-season-episode-graph-depth/CLOSEOUT.md`
- `docs/workstreams/bangumi-relations-and-episode-depth/CLOSEOUT.md`
- `docs/workstreams/douban-subject-kind-precision/CLOSEOUT.md`
- `docs/architecture/LIBRARY_PIPELINE.md`
- `crates/nako-core/src/media/candidate.rs`
- `crates/nako-core/src/media/provider.rs`
- `crates/nako-core/src/repository/metadata.rs`
- `crates/nako-metadata/src/confirmation.rs`

## Opening Recon

Local recon on 2026-06-02:

- `MetadataCandidateGraph` is provider-neutral and currently in-memory.
- Provider depth refresh guards prove related graph nodes do not persist child
  Provider Subjects or Provider Mappings during automatic refresh.
- `ProviderMappingStatus` already has `Candidate`, `Accepted`, and `Rejected`
  states, but automatic refresh currently writes accepted root mappings only.
- `HierarchyConfirmationService` accepts explicit hierarchy/provider mapping
  confirmation, which should remain separate from automatic refresh and
  candidate review planning.
- Generated Artifact apply outcome tables are already a separate control-plane
  workflow and must not become the generic candidate review queue.

## MCDR-020 Evidence

Red check:

- `cargo nextest run -p nako-metadata candidate_review metadata_candidate --no-fail-fast`
  failed before implementation because `build_candidate_review_plan` did not
  exist.

Implemented behavior:

- Added provider-neutral `MetadataCandidateReviewPlan`,
  `MetadataCandidateReviewNode`, and `MetadataCandidateReviewRelationship`
  records in `nako-core`.
- Added pure `build_candidate_review_plan` in `nako-metadata`.
- Review plans preserve root and related Provider Subject summaries,
  relationships, and safe Candidate Record metadata.
- Review plans do not include raw provider payload fields or Provider Mapping
  mutation state, and `MCDR-020` added no repository/schema changes.

Green checks:

- `cargo nextest run -p nako-metadata candidate_review metadata_candidate --no-fail-fast`
- `cargo fmt --all -- --check`
- `git diff --check`

## MCDR-030 Evidence

Red check:

- `cargo nextest run -p nako-db candidate_review --no-fail-fast` failed before
  implementation because `NewMetadataCandidateReview`,
  `MetadataCandidateReviewId`, `MetadataCandidateReviewStatus`, and
  `MetadataCandidateReviewRepository` did not exist.

Implemented behavior:

- Added `MetadataCandidateReviewId`, `NewMetadataCandidateReview`,
  `MetadataCandidateReviewRecord`, and `MetadataCandidateReviewStatus` in
  `nako-core`.
- Added `MetadataCandidateReviewRepository` with upsert/get/find/list methods.
- Added `metadata_candidate_reviews` to SQLite and PostgreSQL baselines.
- Stored only serialized `MetadataCandidateReviewPlan` in `plan_json`; raw
  provider response bodies, tokens, proxy URLs, headers, and paths remain out of
  review snapshots.
- Added SQLite and PostgreSQL repository adapters with item/source/source-key
  idempotency.
- Added a SQLite round-trip test proving review snapshots do not create
  Provider Mapping rows.

Green checks:

- `cargo nextest run -p nako-db candidate_review --no-fail-fast`
- `cargo nextest run -p nako-db provider_subjects --no-fail-fast`
- `cargo nextest run -p nako-db baseline_migration --no-fail-fast`
- `cargo nextest run -p nako-db --no-fail-fast`

## MCDR-040 Evidence

Red checks:

- `cargo nextest run -p nako-metadata candidate_review_decision --no-fail-fast`
  failed before implementation because
  `MetadataCandidateReviewDecisionService`,
  `MetadataCandidateReviewDecisionRequest`, and
  `MetadataCandidateReviewDecision` did not exist.
- The same gate then failed while expired pending reviews remained `Pending`
  after an attempted decision.

Implemented behavior:

- Added `set_metadata_candidate_review_status` to
  `MetadataCandidateReviewRepository`, SQLite, PostgreSQL, and `NakoDatabase`.
- Added `MetadataCandidateReviewDecisionService` in `nako-metadata`.
- Accept/reject transitions are idempotent for the same terminal decision.
- Conflicting terminal decisions fail safely.
- Decisions guard `item_id` and optional `expected_updated_at_ms`.
- Expired pending reviews are marked `Expired` and fail the decision.
- The decision service depends only on `MetadataCandidateReviewRepository`, not
  `ProviderMappingRepository`, so it cannot write Provider Mapping rows.

Green checks:

- `cargo nextest run -p nako-metadata candidate_review_decision --no-fail-fast`
- `cargo nextest run -p nako-db candidate_review --no-fail-fast`
- `cargo nextest run -p nako-metadata --no-fail-fast`
- `cargo nextest run -p nako-db --no-fail-fast`

## Notes

- Accepted-review Provider Mapping application remains a follow-on service, not
  part of review status transitions.
- Do not expose raw provider payloads or secrets.

## MCDR-050 Closeout Evidence

Closeout result:

- lane status is `closed`;
- durable review planning, snapshot persistence, and decision transitions are
  shipped;
- Admin/Web provider depth governance is split to
  `proposed:admin-web-provider-depth-governance`;
- accepted-review Provider Mapping application is opened at
  `docs/workstreams/accepted-review-provider-mapping-application/`.

Fresh closeout checks:

- `cargo nextest run -p nako-metadata candidate_review_decision --no-fail-fast`
  passed: 3 tests run, 3 passed.
- `cargo nextest run -p nako-db candidate_review --no-fail-fast` passed: 1
  test run, 1 passed.
- `python -m json.tool docs/workstreams/metadata-candidate-durable-review/WORKSTREAM.json`
  passed.
- JSONL validation for `TASKS.jsonl`, `CAMPAIGNS.jsonl`, and `CONTEXT.jsonl`
  passed.
- `cargo fmt --all -- --check` passed.
- `git diff --check` passed.
