# Provider Review Global Queue Search - Handoff

Status: Active
Last updated: 2026-06-02

## Current State

The lane is opened from `admin-candidate-review-list-navigation` closeout.
Item-scoped Candidate Review discovery/navigation exists, but operators still
need a global Admin queue/search surface for cross-item triage.

`PRGQ-020` added the read-only Admin API global Candidate Review queue route:
`GET /admin/v1/metadata/candidate-reviews`. The route supports `status`,
`provider`, `limit`, and `offset` query fields and returns redaction-safe
summary rows for triage/navigation.

## Active Task

- Task ID: `PRGQ-030`
- Owner: codex
- Files: `web/src/api/admin`, `web/src/features/admin`, `web/src/shell`,
  `web/src/test`, and
  `docs/workstreams/provider-review-global-queue-search`
- Validation: `npm --prefix web run test`; `npm --prefix web run check`;
  `npm --prefix web run build:budget`; browser smoke if a route or navigation
  mode is added; `git diff --check`
- Status: READY
- Evidence: `docs/workstreams/provider-review-global-queue-search/EVIDENCE_AND_GATES.md`

## Decisions Since Opening

- Start with a read-only Admin API global queue before Web.
- Queue rows are summaries for triage/navigation, not full Candidate Review
  detail duplication.
- HTTP must not load broad rows and filter them in memory; the repository owns
  filters and pagination.
- Batch governance, status mutation, apply mutation, Public Client API, and
  related hierarchy application remain out of scope.
- Web queue should use the new global route for discovery, then navigate into
  the existing Candidate Review detail/apply route instead of duplicating the
  detail workflow.

## Blockers

- None for `PRGQ-030`.

## Next Recommended Action

- Run `PRGQ-030`: add Web Admin global Candidate Review queue/search
  navigation backed by the new Admin API route and route into the existing
  detail/apply page.
