# Provider Governance Durable Batch Execution - Handoff

Status: Active
Last updated: 2026-06-02

## Current State

`PGDBE-020` is accepted.

Current task:

- `PGDBE-030`: Admin API create/status boundary for durable Candidate Review
  batches.

Approved campaign:

- `PGDBE-20260602-01`, limited to `PGDBE-020`, is complete.

Draft next campaign:

- `PGDBE-20260602-02`, covering `PGDBE-030` and `PGDBE-040`, remains draft.
  Activate `PGDBE-030` first and do not start execution work until route/status
  tests are green.

## Read First

- `CONTEXT.md`
- `docs/adr/0006-persist-job-inputs-and-explicit-retry-policy.md`
- `docs/adr/0053-application-control-plane-boundary.md`
- `docs/workstreams/provider-governance-bulk-review/CLOSEOUT.md`
- `docs/workstreams/provider-governance-durable-batch-execution/DESIGN.md`
- `docs/workstreams/provider-governance-durable-batch-execution/TODO.md`
- `docs/workstreams/provider-governance-durable-batch-execution/EVIDENCE_AND_GATES.md`

## Preserved Boundaries

- Do not execute batches in `PGDBE-030`.
- Do not add Web UI in `PGDBE-030`.
- Do not reuse Generated Artifact apply outcome tables.
- Do not add related hierarchy application or child Provider Mapping writes.
- Do not add Public Client API exposure.
- Do not add raw background execution.

## Implementation Hint

The durable batch state now exists behind `MetadataCandidateReviewRepository`.
`PGDBE-030` should create a server/Admin boundary that plans and persists a
queued batch, then reads a redaction-safe status view. It must not execute
Provider Mapping writes during create or status reads.

Expected first files to inspect before editing:

- `crates/nako-api/src`
- `crates/nako-server/src/app/metadata.rs`
- `crates/nako-server/src/routes/admin`
- `crates/nako-core/src/media/candidate.rs`
- `crates/nako-core/src/repository/metadata.rs`
- `crates/nako-db/src/contract_tests.rs`

## Validation

Preferred:

- `cargo nextest run -p nako-server metadata_candidate_review_batch --no-fail-fast`

Fallback used in this environment when `nextest` is unavailable:

- `cargo test -p nako-server metadata_candidate_review_batch -- --nocapture`
- `cargo test -p nako-api admin_contract -- --nocapture`
- `cargo fmt --all -- --check`
- `git diff --check`
