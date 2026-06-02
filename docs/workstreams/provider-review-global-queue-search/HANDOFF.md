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

`PRGQ-030` added Web Admin global Candidate Review queue navigation. The Web
surface can now browse global queue rows, filter by status/provider, keep
pagination in route state, and navigate into the existing detail/apply page.

## Active Task

- Task ID: `PRGQ-040`
- Owner: planner
- Files: `docs/workstreams/provider-review-global-queue-search`,
  `docs/architecture`, `docs/GOALS.md`, and `docs/ROADMAP.md`
- Validation: fresh gate evidence in `EVIDENCE_AND_GATES.md`; JSON/JSONL
  validation; `git diff --check`
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
- The first global queue scope is complete. Batch governance, status mutation,
  related hierarchy application, and provider endpoint depth should be split as
  explicit follow-ons or deferred during closeout.

## Blockers

- None for `PRGQ-040`.

## Next Recommended Action

- Run `PRGQ-040`: close this lane or split follow-ons. Do not add runtime
  behavior under the closeout task.
