# Provider Governance Durable Batch Execution - Milestones

Status: Active
Last updated: 2026-06-02

## M0 - Scope And Evidence Freeze

Exit criteria:

- workstream docs exist and agree;
- `PGDBE-010` is accepted;
- architecture maps route this lane as active;
- `PGDBE-020` is the current ready task.

Status: Done.

## M1 - Durable Batch State And Repository Contract

Exit criteria:

- Candidate Review durable batch, item, status, and execution summary records
  exist in `nako-core`;
- a new explicit job kind/resource class is defined;
- repository trait methods support idempotent commit, lookup, status
  transition, and item outcome commit;
- SQLite and PostgreSQL implementations pass shared contract tests.

Status: Current.

## M2 - Admin Create And Status Boundary

Exit criteria:

- Admin can create a durable Candidate Review batch from selected review IDs;
- create is idempotent and persists a queued job;
- status reads return redacted batch/item summaries;
- no execution occurs during create/status reads.

Status: Pending.

## M3 - Job-Backed Execution

Exit criteria:

- queued batches execute through `DurableJobRuntime`;
- each item calls the single-review application service;
- outcomes are persisted as applied/noop/skipped/stale/conflict/failed;
- cancellation checkpoints are covered;
- no raw `tokio::spawn` path is introduced.

Status: Pending.

## M4 - Web Admin Durable Status

Exit criteria:

- Web Admin can queue a durable batch from selected reviews;
- Web can display queued/running/completed/failed/cancelled status and
  redaction-safe partial results;
- route-state tests and browser smoke pass.

Status: Pending.

## M5 - Closeout

Exit criteria:

- fresh gates are recorded;
- architecture maps route shipped behavior as evidence;
- unfinished scope is split to focused follow-ons.

Status: Pending.
