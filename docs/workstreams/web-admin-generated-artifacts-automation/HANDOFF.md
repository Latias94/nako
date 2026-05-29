# Web Admin Generated Artifacts Automation - Handoff

Status: Active
Last updated: 2026-05-28

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

The old `apps/admin-web` implementation is prior art, not code to copy into the
new shell. The new work belongs in `web/src/api/admin`,
`web/src/features/admin`, `web/src/shell/nako-router.tsx`, and tests under
`web/src/test`.

## Active Task

- Task ID: WAGA-020
- Owner: Codex
- Status: READY
- Validation: Admin data-source contract test, TypeScript check, and updated
  `ROUTE_API_READINESS.md`.

## Next Recommended Action

Start WAGA-020. Audit the exact generated Admin DTOs, then decide the first
implementation slice: read-only proposal route first, guarded review actions
only after review-plan display is proven.
