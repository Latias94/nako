# Watch-folder Suppression Completion Reconciliation Admission

## Goal

When a planned Library File Write suppression completes with `ReconcileScope`,
Nako should immediately hand the affected Media Library back to the existing
library scan admission path. This closes the current gap where the completion
only reports reconciliation intent and waits for later watch-folder ticks to
observe stable candidates again.

## What I Already Know

* `WatchFolderSuppressionAppService::complete_planned_write_suppression` removes
  the active suppression and returns `reconciliation_requested = true` for
  `PlannedWatchFolderWriteCompletion::ReconcileScope`.
* `WatchFolderRuntimeAppService::tick_library` already suppresses planned writes
  and reports `ReconciliationPending` while the suppression is active.
* After completion, existing tests prove the next two runtime ticks can observe
  the source again and eventually enqueue a scan, but completion itself does not
  perform the handoff.
* `LibraryScanAppService::admit_watch_folder_library_scan` is the existing
  bounded/idempotent scan admission seam for watch-folder-triggered scans. It
  reuses queued/running library scans and schedules queued work.
* The suppression service is currently a process-local state service. It should
  stay focused on suppression lifecycle and not own durable scan admission.

## Requirements

* Add an app-level orchestration seam for completing a planned watch-folder write
  suppression and admitting reconciliation work when requested.
* Preserve existing suppression behavior:
  * unknown or expired tokens still return `None`;
  * `SuppressOnly` completion removes the suppression but does not enqueue or
    reuse a library scan;
  * `ReconcileScope` completion removes the suppression and requests a library
    scan through `admit_watch_folder_library_scan`.
* Reuse the existing watch-folder library scan admission semantics:
  * create one `JobKind::LibraryScan` job when no incomplete scan exists;
  * reuse queued/running incomplete scans for the same library;
  * keep execution delegated to `schedule_queued_library_scans`.
* Keep the durable job and diagnostics redaction boundary unchanged. Do not
  expose raw roots, Source Locators, filenames, tokens, paths, `input_json`, or
  `summary_json`.
* Do not change Admin API DTOs, public routes, generated contracts, database
  schema, runtime scheduling loops, or frontend behavior in this slice.

## Acceptance Criteria

* [x] App tests prove `ReconcileScope` suppression completion admits exactly one
  library scan through the existing watch-folder admission path.
* [x] App tests prove a second `ReconcileScope` completion reuses an existing
  queued/running scan instead of inserting a duplicate.
* [x] App tests prove `SuppressOnly` completion performs no scan admission.
* [x] App tests prove unknown/expired completion remains non-mutating.
* [x] Existing watch-folder runtime suppression tests still pass.
* [x] Focused formatting and `nako-server` checks pass.

## Technical Approach

Keep `WatchFolderSuppressionAppService` as the state authority and add a thin
workflow method on `NakoApp` that composes suppression completion with
`LibraryScanAppService::admit_watch_folder_library_scan`. The method should
return the existing completion diagnostic plus an optional admission outcome.
Tests can call this app-level seam directly without changing Admin routes.

## Decision (ADR-lite)

**Context**: Suppression completion currently returns intent, but the caller has
to decide the reconciliation handoff. There is no current external route for
completion; tests call the suppression service directly.

**Decision**: Add the first orchestration seam at the app composition layer
rather than moving scan admission into the suppression state service or creating
new API/DTOs.

**Consequences**: The state service stays simple and reusable. Future NFO,
subtitle, managed import, or Addon-owned Library File Write flows can call the
workflow seam when they complete a write. A future public/Admin route can wrap
the same seam if needed.

## Out of Scope

* New Admin or Public Client route for suppression completion.
* New DTOs or generated SDK updates.
* Database persistence for suppression records.
* OS filesystem watcher integration.
* Reconciliation scheduler policy beyond one library scan admission.
* Frontend changes.

## Technical Notes

* Relevant docs:
  * `CONTEXT.md`
  * `docs/architecture/LIBRARY_PIPELINE.md`
  * `docs/architecture/CONTROL_PLANE.md`
  * `.trellis/spec/nako-server/backend/directory-structure.md`
  * `.trellis/spec/nako-server/backend/quality-guidelines.md`
* Relevant code:
  * `crates/nako-server/src/app/watch_folder_suppression.rs`
  * `crates/nako-server/src/app/watch_folder_runtime.rs`
  * `crates/nako-server/src/app/jobs.rs`
  * `crates/nako-server/src/app/tests/startup.rs`
