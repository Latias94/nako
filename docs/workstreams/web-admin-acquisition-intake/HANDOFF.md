# Web Admin Acquisition Intake - Handoff

Status: Active
Last updated: 2026-05-28

## Current State

This lane is open. WDRP mapped the removed Downloads placeholder to Admin
Acquisition Intake, and the generated Admin contract already exposes the first
read-only candidate list route:

- `ADMIN_API_ROUTES.acquisitionIntakeCandidates`
- `AdminAcquisitionIntakeCandidatesQuery`
- `AdminAcquisitionIntakeCandidateListResponse`
- `AdminAcquisitionIntakeCandidateDiagnostic`

The old `apps/admin-web` implementation is prior art, not code to copy into the
new shell. The new work belongs in `web/src/api/admin`,
`web/src/features/admin`, `web/src/shell/nako-router.tsx`, and tests under
`web/src/test`.

## Active Task

- Task ID: WAAI-020
- Owner: Codex
- Status: READY
- Validation: Admin data-source contract test, TypeScript check, and updated
  `ROUTE_API_READINESS.md`.

## Next Recommended Action

Start WAAI-020. Audit the exact generated Admin DTOs and add the read-model
contract plan before implementing the route.
