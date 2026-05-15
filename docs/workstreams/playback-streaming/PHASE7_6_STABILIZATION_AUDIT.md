# Phase 7.6 Stabilization Audit

## Summary

M7 is complete when the remote playback hardening objective is covered by
runtime implementation, tests, docs, and workspace validation. This audit maps
the goal criteria to concrete artifacts.

## Completion Checklist

### Remote direct play does not buffer full bodies through `Vec<u8>`

Evidence:

- `crates/taru-vfs/src/lib.rs` defines `ReadStream` and
  `StorageBackend::stream_range`.
- `crates/taru-vfs/src/webdav.rs` implements WebDAV `stream_range` with the
  HTTP response byte stream.
- `crates/taru-server/src/app/playback.rs` returns
  `DirectPlaySourceBody::Stream` for remote sources without local path hints.
- `crates/taru-server/src/http/playback.rs` proxies streamed bodies into axum
  responses.
- Tests cover WebDAV byte streaming, direct stream route proxying, and HEAD
  preflight behavior.

### Playback app/http code is split from the largest server files

Evidence:

- Playback planning lives in `crates/taru-server/src/app/playback.rs`.
- Playback HTTP response construction lives in
  `crates/taru-server/src/http/playback.rs`.

Known follow-up:

- Broader `taru-server::app`, `taru-server::http`, and `taru-db` domain splits
  remain M8 modularization work.

### Staged remote inputs have manifest, disk budget, and cleanup

Evidence:

- `crates/taru-core/src/staging.rs` defines staging purpose, state, and record
  models.
- `crates/taru-core/src/repository.rs` defines `StagingManifestRepository`.
- `crates/taru-db/migrations/0014_staging_manifest.sql` persists staging
  manifests.
- `crates/taru-db/src/staging.rs` implements the staging repository.
- `crates/taru-server/src/app/staging.rs` records staged remote inputs,
  enforces `[staging].max_bytes`, computes expiration, and cleans expired
  staged files while preserving active leases.
- Probe and FFmpeg staging paths wrap VFS backends with
  `ManifestRecordingStorageBackend`.

### Playback/storage errors have stable HTTP mapping

Evidence:

- `crates/taru-server/src/http.rs` maps staging budget, staging validation,
  storage timeout, storage unauthorized, storage rate limit, and FFmpeg errors
  to stable public codes.
- `docs/api/HTTP_API.md` documents the playback error summary.
- `api_errors_map_playback_storage_categories` verifies representative public
  codes and messages.

Known follow-up:

- A typed playback/storage error enum would be cleaner than message-pattern
  classification, but M7 has stable public mapping.

### Remote streaming/staging have independent resource budgets

Evidence:

- `crates/taru-server/src/config.rs` defines
  `[playback].remote_stream_concurrency` and
  `[playback].remote_stage_concurrency`.
- `TaruApp` owns separate remote stream and remote stage semaphores.
- Remote direct-play holds a stream permit for the response body plan.
- Remote probe and FFmpeg staging acquire stage permits through
  `ManifestRecordingStorageBackend`.
- Tests cover stream permit lifetime and stage budget waiting.

### Multi-library / multi-remote backend config is clean and formed

Evidence:

- `TaruServerConfig` uses `libraries: Vec<LocalLibraryConfig>` as the only
  library configuration field.
- `[[libraries]]` supports local libraries and per-library `[libraries.webdav]`
  remote backend configuration.
- Startup upserts every configured library.
- `MediaSource` carries `library_id`, and app playback/staging resolves the
  backend from that source identity.
- Tests cover multi-library config parsing, mixed local/WebDAV libraries,
  startup library registration, and remote direct-play backend resolution.

### NFO import/export uses the VFS backend boundary

Evidence:

- `run_nfo_import` and `run_nfo_export` build backends with
  `storage_backend_for_library_root`.
- NFO export checks `StorageCapabilities::WRITABLE`.
- Tests cover WebDAV import and read-only WebDAV export rejection.

### M7 stabilization docs and workspace validation pass

Evidence:

- M7 workstream docs include phase notes through this stabilization audit.
- `docs/development/LOCAL_SETUP.md` documents `[[libraries]]`, WebDAV,
  staging, cleanup, and playback budgets.
- `docs/api/HTTP_API.md` documents remote storage/playback limitations and
  playback error mapping.
- Full validation commands are listed below.

## Validation Commands

Run before closing M7:

```powershell
cargo fmt --all -- --check
cargo check --workspace
cargo check --workspace --tests
cargo nextest run --workspace
git diff --check
```

## Known Limitations After M7

- WebDAV remains read-only; remote NFO export is rejected unless a backend
  advertises writable capabilities.
- Remux and HLS still stage full remote objects before FFmpeg.
- Cleanup is startup-driven; a recurring background cleanup worker can be added
  if operational evidence needs it.
- Playback/storage error classification is stable publicly but not yet a typed
  internal taxonomy.
- Broader server/db file decomposition should be the next modularization goal.
