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

WAAI-030 implemented `/admin/acquisition/intake` in the new `web/` shell. The
route is read-only, Admin-only, and owns `library_id`, `state`, `source_kind`,
`managed_import_artifact_id`, `limit`, and `offset` search params. The page
renders redacted candidate diagnostics, fixture/live source status, pagination,
and redaction-safe fields only.

The old `apps/admin-web` implementation is prior art, not code to copy into the
new shell. The new work belongs in `web/src/api/admin`,
`web/src/features/admin`, `web/src/shell/nako-router.tsx`, and tests under
`web/src/test`.

## Active Task

- Task ID: WAAI-040
- Owner: Codex
- Status: READY
- Validation: mutation boundary decision recorded; if code changes are made,
  `npm --prefix web run test` and `npm --prefix web run check`.

## Next Recommended Action

Start WAAI-040. Decide whether watch-folder discovery belongs in this lane as a
guarded mutation or should split into a follow-on. Do not imply promotion,
apply, or direct library writes.
