# Provider Governance Durable Batch Execution - TODO

Status: Active
Last updated: 2026-06-02

## M0 - Scope And Evidence Freeze

- [x] PGDBE-010 [owner=planner] [deps=none] [scope=docs/workstreams/provider-governance-durable-batch-execution,docs/architecture,docs/GOALS.md,docs/ROADMAP.md]
  Goal: Open the durable Candidate Review batch execution lane after PGBR closeout and freeze the first execution boundary.
  Validation: Workstream docs exist and agree; JSON/JSONL ledgers parse; architecture maps route this lane as active.
  Evidence: `DESIGN.md`, `WORKSTREAM.json`, `EVIDENCE_AND_GATES.md`, `JOURNAL/2026-06-02-PGDBE-010.md`
  Context: `CONTEXT.jsonl`
  Handoff: DONE. Continue with `PGDBE-020`.
  State: `TASKS.jsonl` entry `PGDBE-010` is accepted.

## M1 - Durable Batch State And Repository Contract

- [ ] PGDBE-020 [owner=codex] [deps=PGDBE-010] [scope=crates/nako-core,crates/nako-db,docs/workstreams/provider-governance-durable-batch-execution]
  Goal: Add Candidate Review durable batch records, item statuses, execution summary, job kind/resource class, repository trait methods, SQLite/PostgreSQL schema support, and contract tests.
  Validation: `cargo test -p nako-db metadata_candidate_review_batch -- --nocapture`; `cargo check -p nako-core -p nako-db --tests`; `cargo fmt --all -- --check`; `git diff --check`.
  Review: Repository contracts must prove idempotent batch commit, lookup by idempotency key, status transitions, and per-item outcome updates without touching Provider Mapping state.
  Evidence: `EVIDENCE_AND_GATES.md`, repository contract tests, migration/schema files.
  Context: `CONTEXT.jsonl`
  Handoff: Final status must be DONE, DONE_WITH_CONCERNS, BLOCKED, or NEEDS_CONTEXT.
  State: `TASKS.jsonl` entry `PGDBE-020` records owner, scope, validation, evidence, and handoff status.

## M2 - Admin Create And Status Boundary

- [ ] PGDBE-030 [owner=unassigned] [deps=PGDBE-020] [scope=crates/nako-api,crates/nako-server,docs/workstreams/provider-governance-durable-batch-execution]
  Goal: Add server app and Admin API create/status routes for durable Candidate Review batches, returning queued batch state and redacted item plan snapshots.
  Validation: `cargo test -p nako-server metadata_candidate_review_batch -- --nocapture`; `cargo test -p nako-api admin_contract -- --nocapture`; `cargo fmt --all -- --check`; `git diff --check`.
  Review: Create must be idempotent by key, must persist a job, must reject empty/duplicate/oversized input, and must not execute Provider Mapping writes.
  Evidence: `EVIDENCE_AND_GATES.md`, system route tests, generated Admin contract sync.
  Context: `CONTEXT.jsonl`
  Handoff: Stop before execution if runtime resource policy or job semantics are unclear.
  State: `TASKS.jsonl` entry `PGDBE-030` records status and evidence.

## M3 - Job-Backed Execution

- [ ] PGDBE-040 [owner=unassigned] [deps=PGDBE-030] [scope=crates/nako-server,crates/nako-core,crates/nako-db,docs/workstreams/provider-governance-durable-batch-execution]
  Goal: Execute queued durable Candidate Review batches through `DurableJobRuntime`, recording per-item applied/noop/skipped/stale/conflict/failed outcomes and cancellation checkpoints.
  Validation: `cargo test -p nako-server metadata_candidate_review_batch -- --nocapture`; `cargo test -p nako-metadata candidate_review_application -- --nocapture`; `cargo fmt --all -- --check`; `git diff --check`.
  Review: Execution must call `MetadataCandidateReviewApplicationService` per item, map resource class through metadata shared budget, and must not use raw `tokio::spawn`.
  Evidence: `EVIDENCE_AND_GATES.md`, app/runtime tests.
  Context: `CONTEXT.jsonl`
  Handoff: Split scheduler priority, retry policy expansion, or audit/undo if it grows beyond this lane.
  State: `TASKS.jsonl` entry `PGDBE-040` records status and evidence.

## M4 - Web Admin Durable Batch Status

- [ ] PGDBE-050 [owner=unassigned] [deps=PGDBE-040] [scope=web/src/api/admin,web/src/features/admin,web/src/test,web/scripts,docs/workstreams/provider-governance-durable-batch-execution]
  Goal: Let Web Admin create a durable Candidate Review batch from selected reviews, navigate to or render batch status, and poll redaction-safe results.
  Validation: `npm --prefix web run check`; `npm --prefix web run test`; `npm --prefix web run build:budget`; browser smoke if route behavior changes; `git diff --check`.
  Review: Web must distinguish queued/running/completed/failed/cancelled status and must not render raw idempotency keys, provider payloads, tokens, local paths, or source fingerprints.
  Evidence: `EVIDENCE_AND_GATES.md`, route-state tests, browser smoke.
  Context: `CONTEXT.jsonl`
  Handoff: Keep Public Client API and audit/undo split.
  State: `TASKS.jsonl` entry `PGDBE-050` records status and evidence.

## M5 - Closeout

- [ ] PGDBE-060 [owner=planner] [deps=PGDBE-050] [scope=docs/workstreams/provider-governance-durable-batch-execution,docs/architecture,docs/GOALS.md,docs/ROADMAP.md]
  Goal: Close the durable batch execution lane or split unfinished backend/Web/status/audit follow-ons explicitly.
  Validation: JSON/JSONL validation; fresh gate evidence in `EVIDENCE_AND_GATES.md`; `git diff --check`.
  Review: Verify workstream compliance and code quality before closeout.
  Evidence: `CLOSEOUT.md`, `WORKSTREAM.json`, `EVIDENCE_AND_GATES.md`
  Context: `CONTEXT.jsonl`
  Handoff: Final state must route remaining scope to focused follow-ons.
  State: `TASKS.jsonl` entry `PGDBE-060` records closeout status.
