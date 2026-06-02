# Admin Candidate Review List Navigation - Handoff

Status: Active
Last updated: 2026-06-02

## Current State

The lane is opened from `admin-web-provider-depth-governance` closeout. Durable
Candidate Review detail/apply exists, and `ACRN-020` added the read-only
Admin API item-scoped Candidate Review list route needed for discovery. Web
navigation still needs to consume that route and route into the existing
detail/apply page.

## Active Task

- Task ID: `ACRN-030`
- Owner: codex
- Files: `web/src/api/admin`, `web/src/features/admin`, `web/src/shell`,
  `web/src/test`, and
  `docs/workstreams/admin-candidate-review-list-navigation`
- Validation: `npm --prefix web run test`; `npm --prefix web run check`;
  `npm --prefix web run build:budget`; browser smoke if a route or navigation
  mode is added; `git diff --check`
- Status: READY
- Evidence: `docs/workstreams/admin-candidate-review-list-navigation/EVIDENCE_AND_GATES.md`

## Decisions Since Opening

- Start item-scoped before global queue/search.
- List rows are summaries for triage/navigation, not full Candidate Review
  detail duplication.
- ACRN-020 is read-only and must not write Provider Subject, Provider Mapping,
  Canonical Metadata, or related graph hierarchy state.
- ACRN-020 exposes list rows as summaries, not full detail duplication.
- Web navigation waits for ACRN-030 and should route into the existing
  detail/apply page instead of adding another apply path.

## Blockers

- None for `ACRN-030`.

## Next Recommended Action

- Run `ACRN-030`: add Web Admin item-scoped Candidate Review list/navigation
  using `NAKO_ADMIN_ROUTES.metadataCandidateReviewsForItem` and route into the
  existing detail/apply page.
