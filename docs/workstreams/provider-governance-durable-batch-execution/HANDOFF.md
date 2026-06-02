# Provider Governance Durable Batch Execution - Handoff

Status: Closed
Last updated: 2026-06-02

## Current State

`PGDBE-060` is accepted. The workstream is closed.

Current task:

- None.

Completed campaigns:

- `PGDBE-20260602-01`, limited to `PGDBE-020`, is complete.
- `PGDBE-20260602-02`, covering `PGDBE-030` and `PGDBE-040`, is complete.
- `PGDBE-20260602-03`, covering `PGDBE-050`, is complete.

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
- Do not add Public Client API or related hierarchy application during
  closeout.
- Do not reuse Generated Artifact apply outcome tables.
- Do not add related hierarchy application or child Provider Mapping writes.
- Do not add Public Client API exposure.
- Do not add raw background execution.

## Final State

The durable batch state, Admin create/status boundary, backend execution, and
Web Admin durable create/status workflow now exist. Remaining scope is split to
follow-ons for Public Client API, audit/undo, provider endpoint breadth,
related hierarchy application, and scheduler priority policy.

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
- local HTTP route smoke if browser tooling is unavailable
- `git diff --check`

For `PGDBE-060`:

- JSON/JSONL validation for the workstream ledgers
- `git diff --check`

## Follow-Ons

- `proposed:provider-review-related-hierarchy-application`
- `proposed:douban-tv-episode-endpoint-depth`
- `proposed:provider-review-public-client-governance`
- `proposed:provider-governance-audit-and-undo`
- `proposed:durable-job-priority-policy-and-scheduler-migration`
