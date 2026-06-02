# Provider Governance Durable Batch Execution - Handoff

Status: Active
Last updated: 2026-06-02

## Current State

The lane is open after `PGDBE-010`.

Current task:

- `PGDBE-020`: core/DB durable batch state and repository contract.

Approved campaign:

- `PGDBE-20260602-01`, limited to `PGDBE-020`.

## Read First

- `CONTEXT.md`
- `docs/adr/0006-persist-job-inputs-and-explicit-retry-policy.md`
- `docs/adr/0053-application-control-plane-boundary.md`
- `docs/workstreams/provider-governance-bulk-review/CLOSEOUT.md`
- `docs/workstreams/provider-governance-durable-batch-execution/DESIGN.md`
- `docs/workstreams/provider-governance-durable-batch-execution/TODO.md`
- `docs/workstreams/provider-governance-durable-batch-execution/EVIDENCE_AND_GATES.md`

## Preserved Boundaries

- Do not add Admin routes in `PGDBE-020`.
- Do not execute batches in `PGDBE-020`.
- Do not add Web UI in `PGDBE-020`.
- Do not reuse Generated Artifact apply outcome tables.
- Do not add related hierarchy application or child Provider Mapping writes.
- Do not add Public Client API exposure.
- Do not add raw background execution.

## Implementation Hint

The durable batch state should resemble the shape of Generated Artifact bulk
apply only at the level of concepts: batch, item, status, execution summary,
job, and item outcome. It must have Candidate Review specific types and tables.

Expected first files to inspect before editing:

- `crates/nako-core/src/job.rs`
- `crates/nako-core/src/media/metadata.rs`
- `crates/nako-core/src/repository/metadata.rs`
- `crates/nako-db/src/sqlite/metadata.rs`
- `crates/nako-db/src/postgres/metadata.rs`
- `crates/nako-db/src/contract_tests.rs`

## Validation

Preferred:

- `cargo nextest run -p nako-db metadata_candidate_review_batch --no-fail-fast`

Fallback used in this environment when `nextest` is unavailable:

- `cargo test -p nako-db metadata_candidate_review_batch -- --nocapture`
- `cargo check -p nako-core -p nako-db --tests`
- `cargo fmt --all -- --check`
- `git diff --check`
