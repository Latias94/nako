# Provider Governance Bulk Review - Handoff

Status: Active
Last updated: 2026-06-02

## Current State

The lane is open from `provider-review-global-queue-search` closeout.
Operators can browse a global Metadata Candidate Review queue, apply a single
accepted review through the existing detail/apply route, and request a bounded
read-only batch application plan for selected review IDs. Backend confirmed
batch apply now exists as a bounded synchronous Admin API route with redacted
partial results.

## Active Task

- Task ID: `PGBR-040`
- Owner: codex
- Files: `web/src/api/admin`, `web/src/features/admin`, `web/src/shell`,
  `web/src/test`, and `docs/workstreams/provider-governance-bulk-review`
- Validation: `npm --prefix web run check`; `npm --prefix web run test`;
  `npm --prefix web run build:budget`; browser smoke; `git diff --check`
- Status: READY
- Evidence: `docs/workstreams/provider-governance-bulk-review/EVIDENCE_AND_GATES.md`

## Decisions Since Opening

- Batch governance starts with a read-only Admin API plan.
- Plan rows must reuse single-review application planning semantics.
- `PGBR-020` shipped the read-only batch plan route and generated Admin
  contracts.
- `PGBR-030` shipped bounded backend batch apply with row-level partial results.
- Web Admin selection/confirmation is now scoped to `PGBR-040`.
- Durable job execution is not required for the current bounded synchronous
  backend route.

## Non-Goals To Preserve

- No Public Client API expansion.
- No related Provider Subject, child Provider Mapping, or Media Item hierarchy
  application.
- No Douban TV/episode endpoint breadth.
- No raw provider/local/secret/idempotency-key leakage.
- No hidden raw `tokio::spawn` batch execution.

## Blockers

- None for `PGBR-040`.

## Next Recommended Action

Run `PGBR-040`: add Web Admin queue selection, batch plan inspection, explicit
confirmation, and partial-result rendering against the shipped Admin API
routes. Stop before related hierarchy application, provider endpoint breadth,
Public Client API changes, or backend execution model changes.
