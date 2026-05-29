# Web Admin Acquisition Intake - Handoff

Status: Closed
Last updated: 2026-05-29

## Current State

This lane is closed. WAAI-020 audited the generated Admin acquisition contracts
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

WAAI-040 decided that watch-folder discovery mutation controls do not belong in
this lane. The generated Admin API route exists, but mutation UI must split to a
future guarded lane with explicit permission, confirmation, idempotency,
redacted result, loading/failure, and no-promotion/no-library-write guarantees.

WAAI-050 closed the lane after final frontend gates, bundle budget, desktop
browser smoke, mobile browser smoke, and closeout documentation passed.

The old `apps/admin-web` implementation is prior art, not code to copy into the
new shell. The new work belongs in `web/src/api/admin`,
`web/src/features/admin`, `web/src/shell/nako-router.tsx`, and tests under
`web/src/test`.

## Active Task

None. This workstream is closed.

## Next Recommended Action

Open the selected follow-on lane. The most direct options are:

- guarded watch-folder discovery mutation controls for Admin Acquisition Intake;
- downloader provider/protocol planning before any Media download surface;
- Managed Import promotion/apply UI after mutation safety is explicit;
- return to WDRP or the next active web Admin route lane.
