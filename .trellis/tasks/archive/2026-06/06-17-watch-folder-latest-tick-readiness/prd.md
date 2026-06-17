# Watch-folder Latest Tick Readiness

## Goal

Make the operator-facing Media Library Scan readiness consume the latest watch-folder runtime tick diagnostics, so a started watcher that is currently degraded, blocked, or waiting for reconciliation can surface actionable scan/intake pressure instead of looking ready.

## What I Already Know

* The backend maturity execution plan U4 says watcher events, stable-candidate intake, scan admission, source hash jobs, duplicate suggestions, and VFS repair should behave as one observable intake/repair workflow.
* `docs/architecture/LIBRARY_PIPELINE.md` says watch-folder MVP foundation has stable-candidate evidence and latest runtime tick reporting, but per-library intake diagnostics and scheduled reconciliation remain follow-ons.
* Admin overview already exposes `AdminOverviewWatchFolderRuntimeSummary` with per-library coverage diagnostics and optional `last_tick`.
* `media_library_scan_readiness_check` currently degrades for scan repair pressure, pending source-hash work, failed/pending/never-completed library scans, and watch-folder coverage gaps, but it does not inspect latest tick statuses.
* Existing readiness tests already cover coverage gaps and ignore disabled watchers.

## Requirements

* Degrade Media Library Scan operator readiness when any started watch-folder runtime has a latest tick with:
  * `degraded`
  * `blocked`
  * `reconciliation_pending`
* Preserve the existing readiness priority order:
  * runtime/source-hash repair pressure
  * source-hash pending work
  * failed library scan posture
  * queued/running/never-completed library scan posture
  * watch-folder runtime tick pressure
  * watch-folder coverage gaps
  * ready
* Ignore disabled watchers and started watchers with no latest tick for this slice.
* Keep route/action behavior bounded to existing operator surfaces:
  * degraded/blocked tick pressure should route operators to Jobs when the problem is active intake/scan work.
  * reconciliation-pending should route operators to System Config / watch-folder configuration context unless a more specific existing action already applies.
* Preserve redaction: readiness checks must not expose raw roots, Source Locators, filenames, tokens, raw failure messages, `input_json`, or `summary_json`.
* Do not change Admin API DTO shape, route inventory, runtime scheduling, or database schema.

## Acceptance Criteria

* [x] Unit tests cover degraded latest tick status degrading Media Library Scan readiness with a stable safe source reason.
* [x] Unit tests cover blocked latest tick status degrading Media Library Scan readiness with a stable safe source reason.
* [x] Unit tests cover reconciliation-pending latest tick status degrading Media Library Scan readiness with a stable safe source reason.
* [x] Unit tests prove ready/idle/suppressed latest tick statuses do not degrade readiness when scan posture is otherwise healthy.
* [x] Existing coverage-gap tests still pass and remain lower priority than latest tick pressure.
* [x] No Admin DTO or generated contract files are changed.
* [x] Focused server tests and formatting pass.

## Technical Approach

Add a small helper under `crates/nako-server/src/http/admin.rs` that scans `startup.watch_folder_runtime.diagnostics` for started watchers with latest tick statuses that require operator attention. Return a safe `(source_reason, attention_count, action)` tuple and call it from `media_library_scan_readiness_check` before coverage-gap detection.

## Decision (ADR-lite)

**Context**: The latest watch-folder runtime tick is already a redaction-safe read model in Admin overview, but operator readiness only consumes watcher startup coverage. That means a watcher can be started but actively degraded without changing the readiness summary.

**Decision**: Reuse the existing `last_tick` DTO and readiness summary instead of adding a new route or DTO.

**Consequences**: This is a low-surface-area backend closure step. It improves self-hosted operator evidence without committing to the future scheduler/reconciliation API shape.

## Out of Scope

* New Admin routes or DTO fields.
* OS filesystem watcher daemon.
* Reconciliation scheduler implementation.
* Durable job resource policy changes.
* Frontend changes.

## Technical Notes

* Relevant docs:
  * `docs/plans/2026-06-16-001-feat-backend-self-hosted-maturity-execution-plan.md`
  * `docs/architecture/LIBRARY_PIPELINE.md`
* Relevant code:
  * `crates/nako-server/src/http/admin.rs`
  * `crates/nako-api/src/admin.rs`
  * `crates/nako-server/src/http/tests/system.rs`
  * `crates/nako-server/src/app/watch_folder_runtime.rs`
