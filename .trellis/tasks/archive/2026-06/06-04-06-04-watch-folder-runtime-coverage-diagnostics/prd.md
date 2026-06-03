# Watch folder runtime coverage diagnostics

## Goal

Expose redaction-safe watch-folder runtime coverage diagnostics through the
existing Admin overview startup summary, so operators can tell which libraries
actually have supervised watcher runtime coverage and why configured libraries
were skipped.

## What I already know

* The long-horizon queue still has an open library/watch-folder follow-on for
  degraded and reconciliation-pending diagnostics.
* `watch_folder_runtime.rs` currently starts supervised runtime tasks only for
  libraries with `realtime_monitor = true` and a local root.
* `tick_library` returns `monitored = false` without a reason when a library is
  missing, disabled, or unsupported.
* `ServerStartupReport` only exposes `watch_folder_runtimes_started`, so Admin
  overview cannot explain skipped realtime libraries.
* Admin intake diagnostics already use redacted `scheme://<redacted>` source and
  root references.
* This slice can stay inside server/API Admin overview DTOs; it does not need a
  new route, schema, durable reconciliation job, or scan behavior.

## Requirements

* Add a typed watch-folder runtime coverage diagnostic with redacted root
  reference, coverage status, and safe reason.
* Record coverage for configured libraries at startup:
  * started for local roots with realtime monitoring enabled;
  * disabled when realtime monitoring is off;
  * unsupported-root when the first root is non-local;
  * missing-root when no parseable root is available.
* Surface coverage diagnostics in the existing Admin overview startup payload.
* Preserve existing `watch_folder_runtimes_started` behavior.
* Keep all root references redacted and avoid raw paths, hostnames, tokens, or
  source locators.
* Add focused app/API/HTTP tests for coverage status and redaction.

## Acceptance Criteria

* [x] Startup report records watch-folder coverage diagnostics for started,
  disabled, unsupported-root, and missing-root cases.
* [x] Admin overview serializes the diagnostics without raw local paths or
  backend locators.
* [x] Existing supervised watcher startup behavior remains unchanged.
* [x] Generated Admin contract is updated if the DTO shape changes.
* [x] Focused `nako-api` and `nako-server` tests pass, plus fmt/diff checks.

## Definition of Done

* Code and tests are committed with a Conventional Commit message.
* Verification evidence is persisted in task files.
* Any reusable redaction or runtime-diagnostics convention is written back to
  `.trellis/spec/` if new.
* Task is archived and session journal is recorded.

## Out of Scope

* No watcher event engine changes.
* No new Admin route.
* No scan enqueue or reconciliation job behavior changes.
* No VFS schema or repository changes.
* No public client contract changes.

## Technical Approach

* Add a coverage diagnostic type to the server watch-folder runtime/startup
  boundary.
* Reuse the same library filtering facts used by `start_enabled_watchers`, but
  preserve skip reasons instead of dropping skipped libraries.
* Map the diagnostics into new Admin overview startup DTO fields.
* Keep redaction simple and consistent with existing intake diagnostics:
  `scheme://<redacted>` for parseable roots and `<redacted>` otherwise.

## Research References

* [`research/current-watch-folder-runtime-coverage.md`](research/current-watch-folder-runtime-coverage.md)
  - current code shape, existing diagnostics, and bounded implementation seam.

## Technical Notes

* Relevant specs:
  * `.trellis/spec/nako-server/backend/index.md`
  * `.trellis/spec/nako-server/backend/directory-structure.md`
  * `.trellis/spec/nako-server/backend/quality-guidelines.md`
  * `.trellis/spec/nako-server/backend/logging-guidelines.md`
  * `.trellis/spec/nako-api/backend/index.md`
  * `.trellis/spec/nako-api/backend/admin-and-public-contracts.md`
  * `.trellis/spec/nako-api/backend/quality-guidelines.md`
* Relevant files:
  * `crates/nako-server/src/app/watch_folder_runtime.rs`
  * `crates/nako-server/src/app/startup.rs`
  * `crates/nako-server/src/app/composition.rs`
  * `crates/nako-server/src/http/admin.rs`
  * `crates/nako-api/src/admin.rs`
  * `crates/nako-api/src/admin_contract.rs`
  * `crates/nako-server/src/app/tests/startup.rs`
  * `crates/nako-server/src/http/tests/system.rs`
