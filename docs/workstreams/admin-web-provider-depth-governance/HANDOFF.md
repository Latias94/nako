# Admin Web Provider Depth Governance - Handoff

Status: Active
Last updated: 2026-06-02

## Current State

The lane is opened from `accepted-review-provider-mapping-application`
closeout. Backend accepted-review root Provider Mapping application exists, and
`AWPDG-020` has added read-only Admin API inspection for durable Candidate
Review detail and application-plan facts. `AWPDG-030` has added the explicit
Admin API apply mutation for accepted Candidate Reviews with stale guards,
idempotency-key fingerprinting, replay visibility, and root-only Provider
Subject / Provider Mapping application.

## Active Task

- Task ID: `AWPDG-040`
- Owner: codex
- Files: `web/src/api/admin`, `web/src/features/admin`, `web/src/test`, and
  this workstream evidence
- Validation: `npm --prefix web run test`; `npm --prefix web run check`;
  `npm --prefix web run build:budget`; browser smoke if a route is added;
  `git diff --check`
- Status: READY
- Evidence: `docs/workstreams/admin-web-provider-depth-governance/EVIDENCE_AND_GATES.md`

## Decisions Since Opening

- Read-only Admin API review/detail/application-plan exposure comes before
  apply mutation.
- Existing Admin Catalog Governance Provider Mapping review routes are
  precedent only; they do not replace durable Candidate Review governance.
- Generated Artifact apply outcome tables remain out of scope.
- Related graph nodes remain preview evidence until a future hierarchy
  application lane.
- Public Client API remains out of scope.
- `AWPDG-020` uses `MetadataCandidateReviewApplicationPlan` as the single
  source for application action/reason/source facts; HTTP does not duplicate
  plan rules.
- `GET /admin/v1/metadata/candidate-reviews/{review_id}` is read-only and
  exposes preview related nodes without applying them.
- `POST /admin/v1/metadata/candidate-reviews/{review_id}/apply` calls
  `MetadataCandidateReviewApplicationService`; HTTP does not duplicate apply
  rules.
- Admin apply responses expose idempotency fingerprints only, never raw
  idempotency keys.
- Admin apply remains root-only; preview related nodes are not persisted as
  hierarchy subjects by this mutation.

## Blockers

- None for `AWPDG-040`.

## Next Recommended Action

- Implement `AWPDG-040`: add Web Admin read/confirm/apply UX for durable
  Candidate Review evidence, plan facts, conflict/noop/replay results, and the
  explicit apply confirmation path.
