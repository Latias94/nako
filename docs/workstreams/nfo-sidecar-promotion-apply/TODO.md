# NFO Sidecar Promotion Apply — TODO

Status: Active
Last updated: 2026-05-21

Task IDs use the `NSPA` prefix.

## M0 — Lane Open

- [x] NSPA-010 [owner=planner] [deps=LAIP-070] [scope=docs/workstreams/nfo-sidecar-promotion-apply]
  Goal: Open the follow-on lane with explicit sidecar apply boundaries,
  non-goals, task order, and gates.
  Validation: workstream docs agree and `WORKSTREAM.json` is valid JSON.
  Evidence: `DESIGN.md`, `TODO.md`, `WORKSTREAM.json`.
  Handoff: Execute NSPA-020 after LAIP closeout or umbrella re-scoring.

## M1 — Durable Sidecar Apply Acceptance And Audit

- [x] NSPA-020 [owner=codex] [deps=NSPA-010,LAIP-070] [scope=crates/taru-core,crates/taru-db]
  Goal: Add sidecar apply IDs, operation/state enums, accepted preview snapshot,
  audit outcome records, repository traits, migrations, and backend-neutral
  contract tests.
  Validation: focused DB contract tests for NFO sidecar apply persistence,
  idempotency-key lookup, state transitions, and redacted audit snapshots.
  Evidence: core domain records, repository trait, SQLite/PostgreSQL migrations,
  and DB contract tests.
  Handoff: Wire server acceptance/replay in NSPA-030.

## M2 — App Service Acceptance And Idempotent Replay

- [x] NSPA-030 [owner=codex] [deps=NSPA-020] [scope=crates/taru-server]
  Goal: Add an app-service command that explicitly accepts a current NFO
  authority preview, records a durable sidecar apply attempt, replays matching
  idempotency keys, rejects mismatched/stale requests, and performs no file or
  metadata mutation yet.
  Validation: focused server tests prove accepted preview snapshot, stale
  preview rejection, idempotent replay, redacted diagnostics, and no VFS write
  or canonical metadata mutation.
  Evidence: `AcceptNfoSidecarApplyRequest`,
  `NfoSidecarApplyAcceptanceDiagnostic`, `accept_sidecar_apply`, and focused
  server tests.
  Handoff: Implement export apply in NSPA-040.

## M3 — Export Sidecar Apply

- [x] NSPA-040 [owner=codex] [deps=NSPA-030,nfo-round-trip-preservation,nfo-storage-write-policy,nfo-sidecar-backup-policy,nfo-backup-retention-diagnostics] [scope=crates/taru-nfo,crates/taru-vfs,crates/taru-server]
  Goal: Apply accepted NFO export by using round-trip preservation and
  VFS-backed backup/atomic write/retention diagnostics. Server code must not
  write raw OS paths directly.
  Validation: focused tests prove create, preservation-aware update,
  backup-required forced update, retention diagnostics, stale sidecar rejection,
  and redacted reports.
  Evidence: `ApplyNfoSidecarApplyRequest`, `apply_sidecar_apply`, server export
  apply tests, and existing `taru-nfo`/`taru-vfs` backup/atomic write tests.
  Handoff: Implement import authority apply in NSPA-050.

## M4 — Import Authority Apply

- [x] NSPA-050 [owner=codex] [deps=NSPA-030,NSPA-040] [scope=crates/taru-core,crates/taru-db,crates/taru-nfo,crates/taru-server]
  Goal: Apply accepted NFO import into canonical metadata, field locks/local
  authority, and hierarchy confirmation while respecting user-locked fields.
  Validation: focused tests prove accepted fields, skipped locked fields,
  conflict reporting, hierarchy confirmation, stale target rejection, and no
  sidecar write during import-only apply.
  Evidence: single-source NFO import apply in `taru-nfo`, accepted import apply
  dispatch in `taru-server`, content-fingerprint preview revalidation, and
  focused server tests for commit, stale content rejection, user locks, and
  hierarchy confirmation.
  Handoff: Add partial-failure rollback/repair gates in NSPA-060.

## M5 — Partial Failure Rollback And Repair

- [ ] NSPA-060 [owner=codex] [deps=NSPA-040,NSPA-050] [scope=crates/taru-server,crates/taru-vfs,crates/taru-db]
  Goal: Inject failures across export write, backup restore, metadata commit,
  audit commit, and retention diagnostics. Prove failed-before-mutation,
  rollback-complete, or repair-pending terminal outcomes.
  Validation: tests with failing storage/repository doubles prove no false
  committed state and no unredacted diagnostics.
  Evidence: failure-injection tests and audit state transitions.
  Progress: 2026-05-21 audit commit failure injection now proves
  repair-pending terminal outcomes after export sidecar write and after import
  metadata mutation. Remaining NSPA-060 coverage still needs export write,
  backup restore/rollback, metadata commit, and retention diagnostic failure
  gates before this task is DONE.
  Progress: 2026-05-21 export write failure injection now proves
  `FailedBeforeMutation` without creating a sidecar or leaking raw path/XML
  diagnostics. Remaining NSPA-060 coverage still needs backup
  restore/rollback, metadata commit, and retention diagnostic failure gates.
  Progress: 2026-05-21 import metadata commit failure injection now proves
  `FailedBeforeMutation` before canonical metadata, field locks, sidecars, or
  operator diagnostics are mutated. Remaining NSPA-060 coverage still needs
  backup restore/rollback and retention diagnostic failure gates.
  Handoff: Decide API/UI/addon exposure in NSPA-070.

## M6 — Closeout And Exposure Split

- [ ] NSPA-070 [owner=planner] [deps=NSPA-060] [scope=docs/workstreams/nfo-sidecar-promotion-apply,docs/workstreams/post-rpd-product-hardening]
  Goal: Close or split the sidecar apply lane and decide whether Admin API,
  Public Client API, Addon side-effect, or UI exposure belongs in follow-on
  workstreams.
  Validation: fresh evidence gates are recorded; parent umbrella points to the
  next lane.
  Evidence: closeout journal, updated handoff, and parent re-score.
  Handoff: Return to `post-rpd-product-hardening` for next lane scoring.
