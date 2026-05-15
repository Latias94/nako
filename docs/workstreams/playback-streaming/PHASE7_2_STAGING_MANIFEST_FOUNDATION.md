# Phase 7.2: Staging Manifest Foundation

Status: active; persistence foundation implemented.

## Goal

Start M7.2 by adding a durable manifest for staged remote inputs without
growing the already large `taru-db/src/lib.rs` implementation file.

## Implemented Shape

- Added `StagingManifestId`.
- Added core staging model:
  - `StagingPurpose` for `probe_input` and `ffmpeg_input`;
  - `StagingState` for `staging`, `ready`, and `failed`;
  - `NewStagingManifestRecord`;
  - `StagingManifestRecord`.
- Added `StagingManifestRepository` with upsert, get, find-by-path, list,
  cleanup-candidate listing, touch, delete, and byte-sum operations.
- Added SQLite migration `0014_staging_manifest.sql`.
- Implemented the repository in `crates/taru-db/src/staging.rs` instead of
  expanding the main DB file further.
- Remote FFmpeg input staging now records `ffmpeg_input` manifest entries after
  a WebDAV input is staged for remux or HLS.
- Remote probe input staging now records `probe_input` manifest entries through
  a server-side `StorageBackend` wrapper, keeping repository writes out of
  `taru-library`.
- Added `[staging].max_bytes` config with a conservative default.
- Probe and FFmpeg remote input staging now check the manifest-backed disk
  budget before creating staged files.
- Added `[staging].retention_ms` and `[staging].cleanup_on_startup`.
- Startup cleanup deletes expired manifest records and staged files while
  preserving entries with active leases.

## Validation

Focused validation:

- `cargo check -p taru-core -p taru-db`
- `cargo nextest run -p taru-db sqlite_store_round_trips_staging_manifest_records`
- `cargo check -p taru-server`
- `cargo nextest run -p taru-server source_path_for_ffmpeg_records_manifest_for_remote_staging ffmpeg_source_path_stages_remote_backend_without_local_path_hint ffmpeg_source_path_reuses_local_path_hint_without_staging`
- `cargo nextest run -p taru-server manifest_recording_backend_records_probe_staging`
- `cargo nextest run -p taru-server manifest_recording_backend_rejects_staging_over_disk_budget`
- `cargo nextest run -p taru-server app_startup_cleans_expired_staging_inputs staging_cleanup_preserves_active_leases`

## Remaining Gaps

- Bounded background cleanup is not implemented yet.
- Active lease acquisition/release is not yet connected to runtime playback
  operations.
- Reuse and validation mismatch tests still need to be added alongside cleanup.
