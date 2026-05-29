# Web Admin Generated Artifacts Automation - Handoff

Status: Active
Last updated: 2026-05-29

## Current State

This lane is open. WDRP mapped the removed AI assistant and Automation
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

The old `apps/admin-web` implementation is prior art, not code to copy into the
new shell. The new work belongs in `web/src/api/admin`,
`web/src/features/admin`, `web/src/shell/nako-router.tsx`, and tests under
`web/src/test`.

## Active Task

- Task ID: WAGA-030
- Owner: Codex
- Status: READY
- Validation: `npm --prefix web run test`; `npm --prefix web run check`;
  `npm --prefix web run build:budget`.

## Next Recommended Action

Start WAGA-030. Implement `/admin/automation/generated-artifacts` as a
read-only Admin route with route-owned `limit` and `offset` state, fixture/live
data-source behavior, and redaction-sensitive rendering.
