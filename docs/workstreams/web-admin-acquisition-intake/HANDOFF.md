# Web Admin Acquisition Intake - Handoff

Status: Active
Last updated: 2026-05-29

## Current State

This lane is open. WAAI-020 audited the generated Admin acquisition contracts
and added the new `web/` read-model boundary:

- `AdminApiClient.getAcquisitionIntakeCandidates(query)`
- `createAdminReadModelsDataSource().loadAcquisitionIntake(query)`
- `ADMIN_ACQUISITION_INTAKE_READ_MODEL_FIXTURE`
- `AdminAcquisitionIntakeReadModel`
- data-source contract tests for query serialization, fixture fallback, and
  redaction of non-contract raw fields

The old `apps/admin-web` implementation is prior art, not code to copy into the
new shell. The new work belongs in `web/src/api/admin`,
`web/src/features/admin`, `web/src/shell/nako-router.tsx`, and tests under
`web/src/test`.

## Active Task

- Task ID: WAAI-030
- Owner: Codex
- Status: READY
- Validation: route contract tests, route-state tests, data-source contracts,
  TypeScript check, and bundle budget.

## Next Recommended Action

Start WAAI-030. Wire `/admin/acquisition/intake` into the shell using the
existing read-model boundary; keep it read-only and Admin-only.
