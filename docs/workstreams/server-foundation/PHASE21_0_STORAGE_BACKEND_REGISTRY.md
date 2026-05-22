# Phase 21.0: Storage Backend Registry

## Summary

M21 makes storage backend ownership a library-scoped runtime boundary. The
server no longer treats storage backends as short-lived helpers created at each
call site. `NakoApp` owns a `StorageBackendRegistry`, and scan, probe,
playback, NFO import/export, and FFmpeg staging all resolve storage through
that registry.

## Runtime Boundary

`crates/nako-server/src/app/storage.rs` is the storage composition boundary for
the server:

- `StorageBackendRegistry` caches one `LibraryStorageBackend` per
  configured `library_id`.
- `LibraryStorageBackend` wraps the concrete VFS backend and owns runtime
  state for that library.
- Local libraries use `LocalFsBackend`.
- WebDAV libraries use `WebDavBackend` wrapped by `CachedStorageBackend`.
- Backend diagnostics are exposed through `/storage/backends` without leaking
  local paths or secrets.

Media sources resolve by `source.library_id`. A source whose library is not
configured is rejected instead of being matched by URI scheme or root. This
keeps multiple local libraries and future remote backends from silently sharing
or stealing each other's runtime state.

## Covered Call Paths

The application routes storage access through registry-backed helpers:

- library scan indexing uses `storage_backend_for_library_root`;
- probe staging wraps the library backend in `ManifestRecordingStorageBackend`;
- direct play resolves the source backend through `storage_backend_for_media_source`;
- remux and HLS source input staging reuse the same source backend;
- NFO import/export uses the configured library backend and checks write
  capability before export.

## Resource and Health State

Per-library runtime state is process-local:

- direct-play stream permits live on `LibraryStorageBackend`;
- remote staging permits live on `LibraryStorageBackend`;
- WebDAV cache state is shared through the registry-owned cached backend;
- health counters track last success, last error, and consecutive failures.

This is intentionally not a distributed lock or multi-process pool. If Nako
later supports multiple server processes sharing one media database, staging
budget reservation and backend health should move to database-backed or
external coordination.

## Validation

Close-out validation:

```powershell
cargo fmt --all -- --check
cargo check -p nako-server --tests
cargo nextest run -p nako-server
git diff --check
```
