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

## Validation

Focused validation:

- `cargo check -p taru-core -p taru-db`
- `cargo nextest run -p taru-db sqlite_store_round_trips_staging_manifest_records`

## Remaining Gaps

- Runtime probe/remux/HLS staging still needs to write manifest records.
- Disk budget configuration and enforcement are not connected yet.
- Cleanup worker and startup cleanup are not implemented yet.
- Active lease management is represented in the manifest schema but not yet
  acquired/released by runtime staging paths.
