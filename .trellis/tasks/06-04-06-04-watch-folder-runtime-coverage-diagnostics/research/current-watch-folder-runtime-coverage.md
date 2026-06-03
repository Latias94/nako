# Current Watch Folder Runtime Coverage

## Observed local code

* `crates/nako-server/src/app/watch_folder_runtime.rs`
  * `start_enabled_watchers` lists only libraries where
    `realtime_monitor = true` and the first root parses as `local`.
  * `tick_library` returns `monitored = false` for missing, disabled, or
    unsupported libraries, but the diagnostic has no reason field.
  * `WatchFolderRuntimeTickDiagnostic` reports runtime outcomes after a tick,
    not startup coverage.
* `crates/nako-server/src/app/startup.rs`
  * `ServerStartupReport` exposes `watch_folder_runtimes_started` only.
* `crates/nako-server/src/app/composition.rs`
  * Starts watch-folder runtimes after startup workflow and injects the started
    count into `ServerStartupReport`.
* `crates/nako-api/src/admin.rs` and `crates/nako-server/src/http/admin.rs`
  * Admin overview startup payload exposes the started count.
* `crates/nako-api/src/admin/intake.rs` and
  `crates/nako-server/src/app/acquisition_intake.rs`
  * Existing watch-folder intake diagnostics use redacted root/source refs and
    avoid raw paths.

## Bounded implementation seam

The first useful slice is a startup coverage diagnostic, not a new watcher
engine or reconciliation workflow. It can reuse the existing runtime selection
predicate and preserve skipped libraries as typed redaction-safe diagnostics.

## Risks and constraints

* Admin overview DTO shape changes require generated Admin contract updates and
  contract tests.
* Root redaction must not leak local paths, WebDAV hosts, or tokens.
* The slice must preserve existing runtime startup behavior; diagnostics should
  explain coverage, not change which tasks run.
