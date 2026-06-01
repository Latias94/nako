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
Subject / Provider Mapping application. `AWPDG-040` has added the Web Admin
direct route for inspecting durable Candidate Review evidence and explicitly
confirming accepted-review apply.

## Active Task

- Task ID: `AWPDG-050`
- Owner: planner
- Files: `docs/workstreams/admin-web-provider-depth-governance`,
  `docs/architecture`, `docs/GOALS.md`, and `docs/ROADMAP.md`
- Validation: fresh gate evidence; JSON/JSONL validation; `git diff --check`
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
- Web Admin Candidate Review apply keeps raw idempotency keys out of the UI,
  shows related nodes as preview-only evidence, and uses fixture mode only as a
  read fallback; fixture mutation remains disabled.
- Static mock Admin dashboard actions, recent activity, DLNA, and transcode
  setting panels were reduced or replaced with planned-API surfaces where no
  durable Admin API contract exists.

## Blockers

- None for `AWPDG-050`.

## Next Recommended Action

- Run `AWPDG-050`: close the lane or split follow-ons for related-node
  hierarchy application, provider endpoint depth, Candidate Review list/search
  navigation, and broader provider governance.
