# Provider Governance Bulk Review - Handoff

Status: Active
Last updated: 2026-06-02

## Current State

The lane is opened from `provider-review-global-queue-search` closeout.
Operators can browse a global Metadata Candidate Review queue and apply a
single accepted review through the existing detail/apply route. Batch governance
is still absent and must start with a read-only plan.

## Active Task

- Task ID: `PGBR-020`
- Owner: codex
- Files: `crates/nako-metadata`, `crates/nako-api`, `crates/nako-server`,
  generated Admin contract output under `web/src/api/admin/generated`, and
  `docs/workstreams/provider-governance-bulk-review`
- Validation: `cargo test -p nako-api admin_contract -- --nocapture`;
  `cargo test -p nako-server metadata_candidate_review -- --nocapture`;
  `cargo test -p nako-metadata candidate_review_application -- --nocapture`;
  `cargo fmt --all -- --check`; `git diff --check`
- Status: READY
- Evidence: `docs/workstreams/provider-governance-bulk-review/EVIDENCE_AND_GATES.md`

## Decisions Since Opening

- Batch governance starts with a read-only Admin API plan.
- Plan rows must reuse single-review application planning semantics.
- Batch mutation waits for `PGBR-030`.
- Web Admin selection/confirmation waits for backend mutation semantics.
- Durable job execution is not assumed; it must be used or split only if
  bounded synchronous confirmation cannot remain honest.

## Non-Goals To Preserve

- No Public Client API expansion.
- No related Provider Subject, child Provider Mapping, or Media Item hierarchy
  application.
- No Douban TV/episode endpoint breadth.
- No raw provider/local/secret/idempotency-key leakage.
- No hidden raw `tokio::spawn` batch execution.

## Blockers

- None for `PGBR-020`.

## Next Recommended Action

Run `PGBR-020`: add the read-only batch application plan route and tests. Stop
before mutation, Web selection, related hierarchy application, or provider
endpoint breadth.
