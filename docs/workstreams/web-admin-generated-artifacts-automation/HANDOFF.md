# Web Admin Generated Artifacts Automation - Handoff

Status: Closed
Last updated: 2026-05-29

## Current State

This lane is closed. WDRP mapped the removed AI assistant and Automation
prototypes to Admin Generated Artifacts / Automation diagnostics. The generated
Admin contract already exposes:

- `ADMIN_API_ROUTES.generatedArtifactProposals`
- `ADMIN_API_ROUTES.generatedArtifactReviewPlan`
- `ADMIN_API_ROUTES.generatedArtifactReview`
- `AdminGeneratedArtifactProposalsQuery`
- `AdminGeneratedArtifactProposalListResponse`
- `AdminGeneratedArtifactReviewPlanResponse`
- `AdminGeneratedArtifactReviewResponse`
- `AdminGeneratedArtifactAcceptancePlan`

WAGA-020 audited those contracts and added the new `web/` read-model boundary:

- `AdminApiClient.getGeneratedArtifactProposals(query)`
- `createAdminReadModelsDataSource().loadGeneratedArtifacts(query)`
- `ADMIN_GENERATED_ARTIFACTS_READ_MODEL_FIXTURE`
- `AdminGeneratedArtifactsReadModel`
- data-source contract tests for query serialization, fixture fallback, and
  redaction of non-contract raw fields

WAGA-030 implemented `/admin/automation/generated-artifacts` in the new `web/`
shell. The route is read-only, Admin-only, and owns `limit` and `offset` search
params. The page renders proposal diagnostics, fixture/live source status,
pagination, and redaction-safe fields only.

WAGA-040 decided that review-plan and accept/reject mutation controls do not
belong in this lane. The generated Admin API routes exist, but mutation UI must
split to a future guarded lane with explicit route shape, permission/readiness
disabled states, confirmation, idempotent replay handling, boundary flag
display, result/error rendering, cache invalidation, and redaction guarantees.

WAGA-050 closed the lane after final frontend gates, bundle budget, desktop
browser smoke, mobile browser smoke, and closeout documentation passed.

The old `apps/admin-web` implementation is prior art, not code to copy into the
new shell. The new work belongs in `web/src/api/admin`,
`web/src/features/admin`, `web/src/shell/nako-router.tsx`, and tests under
`web/src/test`.

## Active Task

None. This workstream is closed.

## Next Recommended Action

Open the selected follow-on lane. The most direct options are:

- guarded Generated Artifact review-plan and accept/reject mutations;
- Automation Provider adapter breadth and local runtime integration;
- metadata-authority apply workflow after Acceptance Workflow boundaries are
  explicit;
- addon task/event diagnostics after Addon runtime contracts need UI breadth.
