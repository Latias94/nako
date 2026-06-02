# Provider Governance Bulk Review - Evidence And Gates

Status: Active
Last updated: 2026-06-02

## Opening Evidence

Source coverage:

- `CONTEXT.md` read for Nako domain terms.
- ADR 0007, ADR 0018, ADR 0021, and ADR 0053 read before opening.
- `docs/GOALS.md`, `docs/ROADMAP.md`, `docs/architecture/LANES.md`, and
  `docs/architecture/LIBRARY_PIPELINE.md` inspected after PRGQ closeout.
- `docs/workstreams/provider-review-global-queue-search/CLOSEOUT.md` read and
  used as the immediate follow-on authority.
- `docs/workstreams/admin-web-provider-depth-governance/CLOSEOUT.md` read and
  used as the single-review Admin/Web governance authority.
- `scripts/workstream_inventory.py`, `scripts/program_status.py`, and
  `scripts/validate_orchestration_state.py` are not present in this checkout;
  a read-only `WORKSTREAM.json` status scan found no active implementation
  workstreams before opening this lane.

Green opening gates for `PGBR-010`:

- `python -m json.tool docs/workstreams/provider-governance-bulk-review/WORKSTREAM.json`
  passed.
- JSONL validation for `TASKS.jsonl`, `CAMPAIGNS.jsonl`, and `CONTEXT.jsonl`
  passed: 5 task records, 2 campaign records, and 13 context records.
- `git diff --check` passed with Git CRLF normalization warnings only.

## PGBR-020 Evidence

Implementation:

- Added `POST /admin/v1/metadata/candidate-reviews/batch-application-plan`.
- Added Admin DTOs for selected review IDs, batch summary counts, and
  redaction-safe per-review plan rows.
- Reused existing single-review Candidate Review application planning for each
  selected review instead of adding a second planning policy.
- Synchronized generated Admin TypeScript contracts in `apps/admin-web` and
  `web`.

TDD evidence:

- RED: `cargo test -p nako-server admin_v1_metadata_candidate_review_batch_plan_is_bounded_redacted_and_read_only -- --nocapture`
  failed before the batch DTO/service/route existed.
- GREEN: the same focused system test passed after implementation.

Green gates:

- `cargo test -p nako-server admin_v1_metadata_candidate_review_batch_plan_is_bounded_redacted_and_read_only -- --nocapture`
  passed.
- `cargo test -p nako-api admin_contract -- --nocapture` passed: 5 tests.
- `cargo test -p nako-server metadata_candidate_review -- --nocapture`
  passed: 6 tests.
- `cargo test -p nako-metadata candidate_review_application -- --nocapture`
  passed: 6 tests.
- `cargo fmt --all -- --check` passed.
- `git diff --check` passed with Git CRLF normalization warnings only.

Local tool note:

- `cargo nextest` is unavailable in this checkout (`cargo` reports no
  `nextest` command), so focused `cargo test` gates were used.

Behavior evidence:

- selected review IDs preserve request order and are capped at 50 rows;
- duplicate review IDs return deterministic `invalid_input`;
- accepted rows can plan as `apply`, while pending rows plan as `skip` with
  `ReviewNotAccepted`;
- all returned row boundaries remain read-only;
- the route writes no Provider Mapping or Provider Subject rows in the system
  test;
- response bodies omit candidate overview and tag strings, preserving the
  existing redaction boundary;
- no batch apply mutation, Web UI, Public Client API, related hierarchy
  application, provider endpoint depth, or durable job execution was added.

## PGBR-030 Evidence

Implementation:

- Added `POST /admin/v1/metadata/candidate-reviews/batch-apply`.
- Added bounded batch apply Admin DTOs with per-row status, redacted error,
  idempotency-key fingerprint, summary counts, and provider mapping result
  fields.
- Deepened `MetadataCandidateReviewApplicationService` with a stale-aware
  `plan` method, then reused that method from `apply`.
- Batch confirmation delegates apply/noop rows to the existing single-review
  application service instead of adding a second metadata apply executor.
- Synchronized generated Admin TypeScript contracts in `apps/admin-web` and
  `web`.

TDD evidence:

- RED: `cargo test -p nako-server admin_v1_metadata_candidate_review_batch_apply_reports_partial_results_redacted -- --nocapture`
  failed before the batch apply DTO/status types existed.
- GREEN: the same focused system test passed after implementation.

Green gates:

- `cargo test -p nako-server admin_v1_metadata_candidate_review_batch_apply_reports_partial_results_redacted -- --nocapture`
  passed.
- `cargo test -p nako-api admin_contract -- --nocapture` passed: 5 tests.
- `cargo test -p nako-server metadata_candidate_review -- --nocapture`
  passed: 7 tests.
- `cargo test -p nako-metadata candidate_review_application -- --nocapture`
  passed: 6 tests.

Behavior evidence:

- batch confirmation is capped at 50 rows and duplicate review IDs are rejected
  as deterministic `invalid_input`;
- rows distinguish `applied`, `noop`, `skipped`, `blocked`, `stale`,
  `conflict`, and `failed`;
- stale guard and item mismatch are preserved per row without aborting the
  whole batch;
- replaying the same batch returns noop/replay counts without creating extra
  Provider Mapping rows;
- only root Provider Subject / Provider Mapping state mutates;
- related episode Provider Subjects remain absent after apply;
- response bodies omit raw idempotency keys, candidate overview/tag strings,
  local paths, source fingerprints, and checkout-local paths;
- no Web UI, Public Client API, related hierarchy application, provider
  endpoint depth, durable job, or hidden background execution was added.

## PGBR-040 Gates

Implementation gates:

- `npm --prefix web run check`
- `npm --prefix web run test`
- `npm --prefix web run build:budget`
- browser smoke for queue selection -> batch plan -> confirmation/result route
- `git diff --check`

Behavior evidence required:

- Web Admin selection is explicit and bounded;
- operators inspect a batch plan before confirmation;
- route state preserves queue context;
- fixture fallback does not claim live mutation success;
- no Public Client API surface or related hierarchy mutation is introduced.
