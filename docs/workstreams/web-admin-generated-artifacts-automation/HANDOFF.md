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

WAGA-030 implemented `/admin/automation/generated-artifacts` in the new `web/`
shell. The route is read-only, Admin-only, and owns `limit` and `offset` search
params. The page renders proposal diagnostics, fixture/live source status,
pagination, and redaction-safe fields only.

WAGA-040 decided that review-plan and accept/reject mutation controls do not
belong in this lane. The generated Admin API routes exist, but mutation UI must
split to a future guarded lane with explicit route shape, permission/readiness
disabled states, confirmation, idempotent replay handling, boundary flag
display, result/error rendering, cache invalidation, and redaction guarantees.

The old `apps/admin-web` implementation is prior art, not code to copy into the
new shell. The new work belongs in `web/src/api/admin`,
`web/src/features/admin`, `web/src/shell/nako-router.tsx`, and tests under
`web/src/test`.

## Active Task

- Task ID: WAGA-050
- Owner: Codex
- Status: READY
- Validation: `npm --prefix web run test`; `npm --prefix web run check`;
  `npm --prefix web run build:budget`; browser smoke; `git diff --check`.

## Next Recommended Action

Start WAGA-050. Close the lane with final evidence and follow-ons for guarded
review mutations, provider adapters, local runtime, and metadata-authority
apply.
