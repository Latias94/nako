# Provider Governance Durable Batch Execution - Handoff

Status: Active
Last updated: 2026-06-02

## Current State

`PGDBE-040` is accepted.

Current task:

- `PGDBE-050`: Web Admin durable Candidate Review batch create/status
  workflow.

Approved campaign:

- `PGDBE-20260602-01`, limited to `PGDBE-020`, is complete.

Completed campaign:

- `PGDBE-20260602-02`, covering `PGDBE-030` and `PGDBE-040`, is complete.

Active campaign:

- `PGDBE-20260602-03`, covering `PGDBE-050`, is active for Web Admin durable
  batch status only.

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
- Do not add Web UI in `PGDBE-040`.
- Do not add Public Client API or related hierarchy application in
  `PGDBE-050`.
- Do not reuse Generated Artifact apply outcome tables.
- Do not add related hierarchy application or child Provider Mapping writes.
- Do not add Public Client API exposure.
- Do not add raw background execution.

## Implementation Hint

The durable batch state, Admin create/status boundary, and backend execution
now exist behind `MetadataCandidateReviewRepository` and `NakoMetadataApp`.
`PGDBE-050` should connect Web Admin selection/confirmation to the durable
create route and render/poll redaction-safe batch status.

Expected first files to inspect before editing:

- `web/src/api/admin`
- `web/src/features/admin`
- `web/src/test`
- `web/scripts`
- `apps/admin-web/src/adminApi/generated/contract.ts`

## Validation

Preferred:

- `cargo nextest run -p nako-server metadata_candidate_review_batch --no-fail-fast`

Fallback used in this environment when `nextest` is unavailable:

- `cargo test -p nako-server metadata_candidate_review_batch -- --nocapture`
- `cargo test -p nako-api admin_contract -- --nocapture`
- `cargo test -p nako-metadata candidate_review_application -- --nocapture`
- `cargo fmt --all -- --check`
- `git diff --check`

For `PGDBE-050`:

- `npm --prefix web run check`
- `npm --prefix web run test`
- `npm --prefix web run build:budget`
- browser smoke if route behavior changes
- `git diff --check`
