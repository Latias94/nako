# Admin Candidate Review List Navigation - Handoff

Status: Active
Last updated: 2026-06-02

## Current State

The lane is ready for closeout/follow-on split. Durable Candidate Review
detail/apply exists, `ACRN-020` added the read-only Admin API item-scoped
Candidate Review list route, and `ACRN-030` added Web Admin item-scoped
list/navigation into the existing detail/apply page.

## Active Task

- Task ID: `ACRN-040`
- Owner: planner
- Files: `docs/workstreams/admin-candidate-review-list-navigation`
- Validation: fresh gate evidence in `EVIDENCE_AND_GATES.md`; JSON/JSONL
  validation; `git diff --check`
- Status: READY
- Evidence: `EVIDENCE_AND_GATES.md`; optional `CLOSEOUT.md`

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

- None for `ACRN-040`.

## Next Recommended Action

- Run `ACRN-040`: close this lane or split follow-ons for global Candidate
  Review queues/search, batch governance, and related hierarchy application.
