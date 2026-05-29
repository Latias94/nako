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

WAAI-040 decided that watch-folder discovery mutation controls do not belong in
this lane. The generated Admin API route exists, but mutation UI must split to a
future guarded lane with explicit permission, confirmation, idempotency,
redacted result, loading/failure, and no-promotion/no-library-write guarantees.

The old `apps/admin-web` implementation is prior art, not code to copy into the
new shell. The new work belongs in `web/src/api/admin`,
`web/src/features/admin`, `web/src/shell/nako-router.tsx`, and tests under
`web/src/test`.

## Active Task

- Task ID: WAAI-050
- Owner: Codex
- Status: READY
- Validation: `npm --prefix web run test`; `npm --prefix web run check`;
  `npm --prefix web run build:budget`; browser smoke; `git diff --check`.

## Next Recommended Action

Start WAAI-050. Close the lane with final evidence and follow-ons for the
guarded discovery mutation, downloader protocols, and Managed Import promotion.
