# Phase 3.5: NFO Discovery, Import, and Export Jobs

## Goal

Make NFO sidecars a first-class local metadata workflow instead of only a codec
round-trip test.

This phase adds a foundation for discovering same-stem `.nfo` files, importing
them into canonical metadata, exporting canonical metadata back to sidecars,
and running those operations through persisted jobs.

## Scope

Implemented:

- VFS text read/write methods for sidecar files.
- Local filesystem text read/write implementation with root-boundary checks.
- Same-stem NFO sidecar locator, for example `movie.mkv` to `movie.nfo`.
- `taru-nfo` service for discovery, import, and export.
- NFO import summaries and export summaries.
- NFO import and export job kinds.
- HTTP routes and CLI commands for import/export jobs.
- Import behavior for `read_only`, `local_first`, and `remote_first`.
- Export behavior for `write_sidecar`.

Out of scope:

- Full Jellyfin/Kodi/Plex NFO compatibility.
- Folder-level `movie.nfo` precedence rules.
- Link creation and link policy.
- Remote backend caches.
- Series, season, and episode NFO codecs.

## Policy Semantics

- `disabled`: NFO import/export is not used.
- `read_only`: import NFO metadata and lock imported fields as `Nfo`; never
  write sidecar files.
- `local_first`: import NFO metadata and lock imported fields as `Nfo`.
- `remote_first`: import NFO only into missing fields; do not create NFO locks.
- `write_sidecar`: export canonical metadata to NFO sidecars.

Existing user locks remain authoritative during import. NFO-owned locks can be
updated by later NFO imports because the sidecar is the authority for those
fields.

## Routes

```text
POST /libraries/{library_id}/nfo/import
POST /libraries/{library_id}/nfo/export
```

Both routes return a queued job. The synchronous CLI commands execute the job
immediately and print the completed job output:

```text
taru-server import-nfo [library_id]
taru-server export-nfo [library_id]
```

## Validation

Required checks:

```text
cargo fmt --all -- --check
cargo check --workspace
cargo nextest run --workspace
```
