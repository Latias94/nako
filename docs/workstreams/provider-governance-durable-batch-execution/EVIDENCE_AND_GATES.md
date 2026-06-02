# Provider Governance Durable Batch Execution - Evidence And Gates

Status: Active
Last updated: 2026-06-02

## Opening Evidence

Source coverage:

- `CONTEXT.md` read for Nako domain terms.
- ADR 0006, ADR 0007, ADR 0018, ADR 0021, and ADR 0053 were used as the
  durable metadata/control-plane authority.
- `docs/GOALS.md`, `docs/ROADMAP.md`, `docs/architecture/LANES.md`,
  `docs/architecture/LIBRARY_PIPELINE.md`, `docs/architecture/CONTROL_PLANE.md`,
  and `docs/architecture/WORKSTREAM_LINKS.md` were inspected after PGBR
  closeout.
- `docs/workstreams/provider-governance-bulk-review/CLOSEOUT.md` split durable
  execution as this follow-on.
- `scripts/workstream_inventory.py`, `scripts/program_status.py`, and
  `scripts/validate_orchestration_state.py` are not present in this checkout;
  manual `WORKSTREAM.json` status parsing found no active/draft workstreams
  before this lane opened.
- A read-only explorer independently recommended this lane as the next
  focused workstream and warned against raw `tokio::spawn`, duplicate apply
  executors, Generated Artifact table reuse, Public Client API expansion, and
  related hierarchy application.

Green opening gates for `PGDBE-010`:

- `python -m json.tool docs/workstreams/provider-governance-durable-batch-execution/WORKSTREAM.json`
- JSONL validation for `TASKS.jsonl`, `CAMPAIGNS.jsonl`, and `CONTEXT.jsonl`
- `git diff --check`

## PGDBE-020 Gate Plan

Implementation scope:

- Candidate Review durable batch types and constants in `nako-core`.
- Candidate Review batch repository trait methods.
- SQLite and PostgreSQL persistence.
- Repository contract tests proving idempotency, status transitions, lookup, and
  per-item outcomes.

Validation:

- `cargo test -p nako-db metadata_candidate_review_batch -- --nocapture`
- `cargo check -p nako-core -p nako-db --tests`
- `cargo fmt --all -- --check`
- `git diff --check`

Acceptance evidence must prove:

- batch commit is idempotent by normalized idempotency key;
- batch items preserve request order and plan snapshots;
- batch status transitions fail closed on invalid expected states;
- item outcome commits update only batch item state;
- job kind/resource class are explicit and ready for runtime mapping;
- no Provider Subject, Provider Mapping, Canonical Metadata, Admin API route,
  Web UI, Public Client API, or related hierarchy state changes are introduced.

## PGDBE-020 Evidence

Implemented:

- Candidate Review durable batch IDs, records, statuses, execution summaries,
  item commits, and item outcome commits in `nako-core`.
- `JobKind::MetadataCandidateReviewBatchApply` and
  `metadata.candidate_review_batch_apply` resource-class binding.
- `MetadataCandidateReviewRepository` batch methods for idempotent commit,
  lookup, status compare-and-set, and item outcome commit.
- SQLite and PostgreSQL baseline tables for Candidate Review batches and
  batch items.
- SQLite/PostgreSQL repository implementations and `NakoDatabase` facade
  delegation.
- Repository contract coverage for idempotent replay, transaction rollback on
  item idempotency conflict, job persistence/rollback, status transition
  failure, item order preservation, and per-item outcome summary updates.

Green gates:

- `cargo test -p nako-db metadata_candidate_review_batch -- --nocapture`
  - Result: passed for SQLite; PostgreSQL contract ignored because
    `NAKO_TEST_POSTGRES_URL` is not set in this environment.
- `cargo check -p nako-core -p nako-db --tests`
  - Result: passed.
- `cargo fmt --all -- --check`
  - Result: passed after running `cargo fmt --all`.
- `git diff --check`
  - Result: passed; Git emitted CRLF normalization warnings only.

## PGDBE-030 Evidence

Implemented:

- Admin API DTOs for durable Candidate Review batch create/status responses,
  including redaction-safe plan snapshots and idempotency-key fingerprints
  without raw idempotency keys.
- `nako-server` metadata app boundary for idempotent queued batch creation,
  per-item plan snapshot persistence, durable job persistence, and status
  lookup.
- Admin HTTP routes:
  - `POST /admin/v1/metadata/candidate-reviews/batches`
  - `GET /admin/v1/metadata/candidate-reviews/batches/{batch_id}`
- System route coverage proving create replay, persisted job facts, status
  lookup, duplicate input rejection, and no Provider Mapping or Provider
  Subject writes during create/status.
- Generated Admin TypeScript contract synchronization for both Admin Web and
  Web contract locations.

Green gates:

- `cargo test -p nako-server metadata_candidate_review_batch -- --nocapture`
  - Result: passed, 3 tests.
- `cargo test -p nako-api admin_contract -- --nocapture`
  - Result: passed, 5 tests.
- `cargo check -p nako-api -p nako-server --tests`
  - Result: passed.
- `cargo fmt --all -- --check`
  - Result: passed.
- `git diff --check`
  - Result: passed; Git emitted CRLF normalization warnings only.

## PGDBE-040 Evidence

Implemented:

- `nako-server` metadata app execution entrypoint for queued Candidate Review
  durable batches.
- Job execution through `DurableJobRuntime` with the Candidate Review batch job
  kind/resource class mapped to the metadata shared runtime budget.
- Per-item execution through `MetadataCandidateReviewApplicationService`,
  preserving the existing stale guard, root-only Provider Subject / Provider
  Mapping application, idempotent noop behavior, and related hierarchy split.
- Per-item outcome persistence for applied, noop, skipped, blocked, stale,
  conflict, and failed states.
- Batch terminal synchronization for completed, cancelled, and infrastructure
  failed execution.
- Route/system coverage proving Admin status reads see completed execution
  summaries and queued job cancellation maps to batch cancellation.

Green gates:

- `cargo test -p nako-server metadata_candidate_review_batch -- --nocapture`
  - Result: passed, 3 tests.
- `cargo test -p nako-metadata candidate_review_application -- --nocapture`
  - Result: passed, 6 tests.
- `cargo check -p nako-server -p nako-metadata --tests`
  - Result: passed.
- `cargo fmt --all -- --check`
  - Result: passed.
- `git diff --check`
  - Result: passed; Git emitted CRLF normalization warnings only.

## Later Gates

`PGDBE-050`:

- Web Admin can create/read/poll durable batches.
- Route-state tests and browser smoke prove no raw provider, secret, path,
  source fingerprint, or raw idempotency-key facts render.

## Tooling Gap

The planner skill references orchestration scripts that are absent from this
checkout. This workstream uses manual JSON/JSONL validation until a separate
tooling follow-on restores or removes those script references.
