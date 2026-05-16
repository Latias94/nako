# Phase 24.1: Server Architecture Hardening Implementation Slice

## Summary

This slice moves M24 from documentation into enforceable server code. The
server now has a runtime supervisor boundary, focused service handles for
catalog, library administration, storage diagnostics, NFO, and extension
workflows, stricter library identity resolution from persisted media sources,
and an atomic repository boundary for catalog graph replacement.

Follow-up passes in this M24 slice also moved metadata and playback orchestration
behind service handles, kept library scan/probe behind `LibraryScanAppService`,
removed the temporary CLI/test root-app forwards, and replaced hand-written NFO
XML walking with a `roxmltree` parser boundary.

## Code Changes

- Added `crates/taru-server/src/app/runtime.rs` with `RuntimeSupervisor`,
  shutdown token ownership, task diagnostics, cancellation, and panic
  accounting.
- Routed library scan jobs, metadata refresh and maintenance jobs, metadata
  lifecycle loops, NFO import/export jobs, and dropped staging lease release
  work through the supervisor.
- Added `AddonAppService`, `AutomationAppService`, `WebhookAppService`,
  `CatalogAppService`, `LibraryAppService`, `StorageDiagnosticsAppService`,
  `MetadataAppService`, `NfoAppService`, `LibraryScanAppService`, and
  `PlaybackAppService` as explicit service handles composed by `TaruApp`.
- Updated addon, automation, webhook, catalog, library, storage diagnostics,
  metadata, playback, and NFO HTTP handlers to call service handles instead of
  root-app convenience methods.
- Changed metadata library resolution to use persisted `MediaSource.library_id`
  via `list_item_sources` and reject items without a persisted source.
- Added `CatalogItemGraphReplacement` and
  `CatalogRepository::replace_item_catalog_graph` so catalog hydration replaces
  item graph records inside one SQLite transaction before rebuilding search
  projection.
- Updated server shutdown to use Axum graceful shutdown and call
  `app.shutdown_runtime()` after serving exits.
- Changed CLI and app tests to use service handles directly, then removed the
  root-app forwarding methods for jobs, library scan, library administration,
  metadata, NFO, and playback workflows.
- Added `roxmltree` to `taru-nfo` and replaced tag-search XML parsing helpers
  with structured node traversal in `MovieNfoCodec`.

## Current Public Surface Audit

`TaruApp` should continue moving toward a composition root. After this slice,
it intentionally still owns:

- startup and configured library validation;
- runtime diagnostics and shutdown;
- service handle accessors for addon, automation, webhook, catalog, library,
  storage diagnostics, jobs, library scan/probe, metadata, NFO, and playback
  workflows.

`TaruApp` no longer exposes workflow forwarding methods for scan, list, NFO,
metadata, playback, or job reads. CLI, HTTP handlers, and app tests call the
focused service handles directly.

## Runtime Audit

Production app code now has one detached spawn boundary:

- `RuntimeSupervisor::spawn`, which wraps `tokio::spawn` and records task
  lifecycle diagnostics.

`tokio::task::JoinSet` remains in webhook delivery as request-scoped structured
concurrency. It is not a detached background worker and completes before the
request returns.

Automation enqueue is synchronous and currently has no detached worker. It does
not need supervisor registration until automation gets an asynchronous runner.

Test-only `tokio::spawn` calls remain in mock servers and concurrency tests.

## Repository Boundary Audit

Completed in this slice:

- Catalog graph hydration no longer clears and writes credits, genres, tags,
  collections, studios, and images through a long app-level sequence.
- `taru-catalog` builds a graph replacement value and delegates atomic writes
  to the repository.
- `taru-db` owns the SQLite transaction and SQL helper functions for the
  replacement.

Still pending:

- App services still depend directly on `SqliteStore` in several places.
  This is intentionally server-local where the service composes multiple
  repository traits on SQLite; future alternate stores should introduce narrow
  ports at the service boundary instead of re-expanding `TaruApp`.

## Obsolete Helper Cleanup

Completed in this slice:

- Removed the old configured-library fallback behavior for metadata refresh.
  An item must have a persisted media source so library identity is explicit.
- Storage backend resolution is keyed by `MediaSource.library_id` instead of
  path inference.
- Replaced hand-written NFO XML walking with `roxmltree` node traversal.
- Removed temporary root-app forwarding methods after CLI and tests moved to
  service handles.
- Audited single-library compatibility paths in the server app surface; no
  obsolete root-app fallback remains.

## Validation

Validated on this slice:

```powershell
cargo fmt --all -- --check
cargo check --workspace --tests
cargo nextest run -p taru-server --no-fail-fast
cargo nextest run -p taru-nfo --no-fail-fast
cargo nextest run -p taru-catalog --no-fail-fast
cargo nextest run -p taru-db catalog metadata scan --no-fail-fast
cargo nextest run --workspace --no-fail-fast
git diff --check
```

Results:

- `taru-server`: 90 tests passed.
- `taru-server` focused metadata/playback/staging/storage/startup/HTTP suites:
  60 tests passed after service-handle migration.
- `taru-nfo`: 7 tests passed after the structured parser migration.
- `taru-catalog`: 2 tests passed.
- `taru-db` focused suites: 9 tests passed.
- Workspace: 229 tests passed.
- `git diff --check`: passed; Git reported CRLF normalization warnings only.

## Close-Out Status

All M24 close-out gates are satisfied as of this slice.
