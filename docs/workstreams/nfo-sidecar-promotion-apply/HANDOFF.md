# NFO Sidecar Promotion Apply — Handoff

Status: Active
Last updated: 2026-05-21

## Current State

This lane is opened as the follow-on split from
`link-apply-and-import-promotion` LAIP-070. The split is intentional: Managed
Import promotion applies staged artifacts into a Media Library, while NFO
sidecar import/export apply is a separate **Library File Write** and
metadata-authority workflow.

Existing prerequisites are in place:

- `taru-nfo` round-trip preservation exists.
- VFS-backed NFO storage write policy exists.
- sidecar backup policy and backup retention diagnostics exist.
- `nfo-link-authority` can produce non-mutating NFO authority previews.
- LAIP promotion apply proves accepted, idempotent, cleanup-aware storage and
  catalog mutation for Media Sources, but it does not mutate sidecars.

## Completed Tasks

- Task ID: NSPA-020
- Owner: codex
- Files: `crates/taru-core`, `crates/taru-db`
- Validation: focused DB contract tests for durable sidecar apply records,
  idempotency-key lookup, state transitions, and redacted audit snapshots.
- Status: DONE
- Evidence: core domain records, repository trait, SQLite/PostgreSQL
  migrations and adapters, facade dispatch, and backend-neutral contract tests.

- Task ID: NSPA-030
- Owner: codex
- Files: `crates/taru-server`
- Validation: focused server tests prove accepted preview snapshot, stale
  preview rejection, idempotent replay, redacted diagnostics, and no VFS write
  or canonical metadata mutation.
- Status: DONE
- Evidence: `AcceptNfoSidecarApplyRequest`,
  `NfoSidecarApplyAcceptanceDiagnostic`, `accept_sidecar_apply`, and focused
  server tests.

- Task ID: NSPA-040
- Owner: codex
- Files: `crates/taru-server`; existing `taru-nfo` and `taru-vfs` export/write
  boundaries
- Validation: focused tests prove create, preservation-aware update,
  backup-required forced update, retention diagnostics, stale sidecar
  rejection, and redacted reports.
- Status: DONE
- Evidence: `ApplyNfoSidecarApplyRequest`, `apply_sidecar_apply`, server
  export apply tests, and existing `taru-nfo`/`taru-vfs` backup/atomic write
  tests.

- Task ID: NSPA-050
- Owner: codex
- Files: `crates/taru-nfo`, `crates/taru-server`
- Validation: focused tests prove accepted import fields, stale import content
  rejection, user-locked field preservation, hierarchy confirmation, and no
  sidecar write during import-only apply.
- Status: DONE
- Evidence: `NfoImportSourceRequest`, `NfoService::import_media_source`,
  content-fingerprint preview snapshots, import dispatch in
  `apply_sidecar_apply`, and focused server tests.

## Active Task

- Task ID: NSPA-060
- Owner: codex
- Files: `crates/taru-server`, `crates/taru-vfs`, `crates/taru-db`
- Validation: tests with failing storage/repository doubles prove no false
  committed state and no unredacted diagnostics across export/import partial
  failures.
- Status: IN_PROGRESS
- Evidence: NSPA-040 export apply and NSPA-050 import apply mutation
  boundaries. The first NSPA-060 slice now injects final audit commit failure
  after export sidecar write and after import metadata mutation; both paths
  record `RepairPending`, replay as terminal diagnostics, and avoid raw path/XML
  leakage. The second NSPA-060 slice injects export write failure through a
  failing storage backend and proves `FailedBeforeMutation` without sidecar
  creation or raw path/XML leakage. The third NSPA-060 slice injects import
  metadata commit failure before canonical mutation and proves
  `FailedBeforeMutation` without metadata, field lock, sidecar, or raw
  diagnostic leakage.

## Decisions

- NFO preview is explanatory only; it is not an authorization token.
- Sidecar apply requires explicit operator or policy acceptance and an
  idempotency key.
- Export sidecar apply must use `taru-nfo` round-trip preservation and VFS
  storage write APIs. Server code must not write raw OS paths directly.
- Import sidecar apply must update canonical metadata, field locks/local
  authority, and hierarchy confirmation only through durable acceptance.
- Backup-required, force-overwrite, retention, field-lock, and hierarchy
  warnings must be part of accepted preview facts.
- Partial failure must produce failed-before-mutation, rollback-complete, or
  repair-pending audit state, not false committed state.
- Operator-facing diagnostics must not leak raw local paths, raw XML, raw
  provider payloads, or secrets.
- Addon side effects and AI-generated artifacts may consume this lane later,
  but they must not bypass it.

## Blockers

- None.

## Next Recommended Action

- Continue NSPA-060 with TDD: add failing storage/repository doubles for backup
  restore or rollback and retention diagnostic failures. Keep proving
  failed-before-mutation, rollback-complete, or repair-pending outcomes instead
  of any false committed state.
