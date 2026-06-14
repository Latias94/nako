# M2 Watcher Incremental Scan Reliability

## Goal

Deepen the first M2 watcher/incremental scan reliability slice by making
watch-folder scan admission evidence explicit and testable. Operators and
future Admin diagnostics should be able to distinguish a runtime tick that
queued a new scan from one that reused an existing queued or running scan, while
the runtime still avoids duplicate scans and never executes scan work inline.

## What I Already Know

- M2 targets large-library reliability: watcher/incremental scan, retries,
  restarts, long-running jobs, and observable queue pressure.
- The watch-folder runtime already exists and is supervised through
  `RuntimeSupervisor`.
- Stable candidate intake already requires repeated unchanged observations
  before a scan is admitted.
- Watch-folder scan admission already coalesces with existing queued/running
  library scans for the same library.
- Current `WatchFolderRuntimeTickDiagnostic` exposes `scan_job_id` and a
  boolean `reused_existing_scan`, but it does not make the admission outcome
  explicit enough for future diagnostics or regression tests.

## Requirements

- Add a typed watch-folder scan admission status to runtime diagnostics.
- Preserve existing behavior:
  - first observation does not enqueue;
  - second unchanged observation can admit a scan;
  - existing queued same-library scan is reused;
  - existing running same-library scan is reused;
  - no scan/probe work is executed inline by the watcher runtime.
- Keep `scan_job_id` for the admitted/reused job ID.
- Keep `reused_existing_scan` for compatibility with existing tests/logs.
- Do not add Admin/Public API, schema migrations, OS filesystem watcher events,
  new worker loops, or broad incremental scan redesign in this slice.
- Keep diagnostics redaction-safe: only status, job ID, counts, and booleans.

## Acceptance Criteria

- [ ] `WatchFolderRuntimeTickDiagnostic` has a typed admission status.
- [ ] No-ready-candidate ticks report a no-admission status.
- [ ] Newly admitted scans report a new-enqueued status.
- [ ] Existing queued scans report a reused-queued status.
- [ ] Existing running scans report a reused-running status.
- [ ] Existing watch-folder runtime tests pass and cover the new status.
- [ ] Specs document the status contract for future Admin diagnostics.

## Definition of Done

- Focused `nako-server` watch-folder runtime tests pass.
- `cargo check -p nako-server --tests` passes.
- Formatting and whitespace checks pass.
- Trellis task context is valid.
- Specs are updated if the diagnostic contract changes.

## Technical Approach

Use the existing internal app-service boundary:

- Extend `LibraryScanAdmissionOutcome` with helper logic that preserves the
  existing job while exposing whether a reused job was queued or running.
- Add a small `WatchFolderScanAdmissionStatus` enum to
  `watch_folder_runtime.rs`.
- Populate the enum in `tick_library` based on the admission outcome.
- Update existing watch-folder runtime tests to assert the typed status.

## Decision (ADR-lite)

**Context**: M2 needs watcher/incremental scan reliability, but the repo already
has the runtime, stable candidate evidence, startup coverage, and queue
coalescing foundation. The next useful slice should not reopen broad watcher
architecture.

**Decision**: Start by turning watch-folder scan admission evidence into a
typed internal diagnostic contract.

**Consequences**: Future Admin diagnostics can reuse this explicit status
without guessing from booleans, while this slice remains small and avoids
schema/API/runtime expansion.

## Out of Scope

- OS filesystem watcher event integration.
- Remote storage watcher semantics.
- New scheduler, executor, or raw `tokio::spawn` loop.
- Admin/Public API changes or generated contract changes.
- Database schema or durable job schema changes.
- Automatic source duplicate reconciliation or source merge behavior.

## Technical Notes

- Relevant architecture:
  - `docs/ROADMAP.md`
  - `docs/architecture/LIBRARY_PIPELINE.md`
  - `docs/architecture/CONTROL_PLANE.md`
  - `docs/architecture/STORAGE_VFS.md`
- Relevant specs:
  - `.trellis/spec/nako-server/backend/directory-structure.md`
  - `.trellis/spec/nako-server/backend/quality-guidelines.md`
  - `.trellis/spec/nako-library/backend/quality-guidelines.md`
- Relevant code:
  - `crates/nako-server/src/app/watch_folder_runtime.rs`
  - `crates/nako-server/src/app/jobs.rs`
  - `crates/nako-server/src/app/tests/startup.rs`
  - `crates/nako-library/src/intake.rs`

## Open Questions

- None for this MVP. The implementation should preserve behavior and make the
  existing admission outcome explicit.
