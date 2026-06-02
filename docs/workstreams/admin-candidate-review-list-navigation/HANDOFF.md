# Admin Candidate Review List Navigation - Handoff

Status: Closed
Last updated: 2026-06-02

## Current State

The lane is closed. Durable Candidate Review detail/apply exists, `ACRN-020`
added the read-only Admin API item-scoped Candidate Review list route,
`ACRN-030` added Web Admin item-scoped list/navigation into the existing
detail/apply page, and `ACRN-040` split remaining queue, batch, and hierarchy
work to follow-ons.

## Closed Task

- Task ID: `ACRN-040`
- Owner: planner
- Files: `docs/workstreams/admin-candidate-review-list-navigation`,
  `docs/architecture`, `docs/GOALS.md`, and `docs/ROADMAP.md`
- Validation: fresh gate evidence in `EVIDENCE_AND_GATES.md`; JSON/JSONL
  validation; `git diff --check`
- Status: DONE
- Evidence: `CLOSEOUT.md`; `EVIDENCE_AND_GATES.md`

## Decisions Since Opening

- Start item-scoped before global queue/search.
- List rows are summaries for triage/navigation, not full Candidate Review
  detail duplication.
- ACRN-020 is read-only and must not write Provider Subject, Provider Mapping,
  Canonical Metadata, or related graph hierarchy state.
- ACRN-020 exposes list rows as summaries, not full detail duplication.
- ACRN-030 routes Web navigation into the existing detail/apply page instead of
  adding another apply path.
- ACRN-030 keeps list rows as navigation/triage summaries and does not add a
  second apply mutation path, batch apply, hierarchy mutation, or Public Client
  API expansion.
- ACRN-030 moved only the aggregate `total-js` gzip budget from 341 KiB to
  343 KiB after a measured 342.05 KiB production build; route-level budgets
  stayed unchanged.

## Blockers

- None. This lane is closed.

## Next Recommended Action

- Open one focused follow-on rather than reopening this lane:
  `proposed:provider-review-global-queue-search`,
  `proposed:provider-governance-bulk-review`, or
  `proposed:provider-review-related-hierarchy-application`.
