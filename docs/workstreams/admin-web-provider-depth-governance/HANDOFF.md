# Admin Web Provider Depth Governance - Handoff

Status: Active
Last updated: 2026-06-02

## Current State

The lane is opened from `accepted-review-provider-mapping-application`
closeout. Backend accepted-review root Provider Mapping application exists, and
`AWPDG-020` has added read-only Admin API inspection for durable Candidate
Review detail and application-plan facts.

## Active Task

- Task ID: `AWPDG-030`
- Owner: codex
- Files: `crates/nako-api`, `crates/nako-server`, `crates/nako-metadata`, and
  this workstream evidence
- Validation: `cargo nextest run -p nako-api admin_contract --no-fail-fast`;
  `cargo nextest run -p nako-server candidate_review admin --no-fail-fast`;
  `cargo nextest run -p nako-metadata candidate_review_application --no-fail-fast`;
  `cargo fmt --all -- --check`; `git diff --check`
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

## Blockers

- None for `AWPDG-030`.

## Next Recommended Action

- Open an `AWPDG-030` mutation campaign and add an explicit Admin API apply
  route that calls `MetadataCandidateReviewApplicationService` with stale
  guards and idempotency.
