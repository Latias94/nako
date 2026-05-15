# Phase 6.5: Remote Storage Stabilization

## Status

Completed.

## Objective

Make the WebDAV remote-storage preview documented, bounded, and ready for
developer testing.

## Implemented

- Added `[library.webdav]` preview configuration:
  - `root` selects the WebDAV storage URI scanned by the configured library;
  - `base_url` points to the WebDAV endpoint and rejects embedded credentials
    through `WebDavBackend`;
  - `username` and `password_env` keep credentials as secret references;
  - `timeout_ms` and `max_attempts` carry bounded remote request policy.
- Wired server storage backend construction:
  - local sources still use `LocalFsBackend`;
  - WebDAV sources use `WebDavBackend` wrapped in `CachedStorageBackend`;
  - configured WebDAV library scan/probe uses the WebDAV root from
    `library_from_config`;
  - remote probe staging uses `remux_staging_root/probe-inputs`.
- Updated HTTP and local setup docs for WebDAV scan, direct play, remux, HLS,
  staging paths, and preview limitations.
- Added an app-level mocked WebDAV test proving server config can build a
  scanner backend for a WebDAV root.

## Validation

- `cargo test -p taru-server config::tests::config_round_trips_from_toml`
- `cargo test -p taru-server app::tests::webdav_preview_config_builds_scanner_backend`
- `cargo fmt --all -- --check`
- `cargo check --workspace`
- `cargo nextest run --workspace` (153 passed)
- `git diff --check`

## Known Limitations

- WebDAV is read-only in M6.
- Only one configured library is supported by `TaruServerConfig`.
- Remote direct play buffers selected range bytes in memory instead of
  streaming an upstream response body.
- Remux and HLS stage full remote objects before FFmpeg.
- Staging has deterministic paths but no disk budget, cleanup worker, or
  persistent manifest yet.
- Remote storage timeout/stale-cache failures still use coarse storage error
  responses at the HTTP boundary.
- NFO import/export remains a local filesystem sidecar workflow.
- S3-compatible storage, rclone providers, UI configuration, and multi-library
  source management are deferred.

## Next Goal Recommendation

Split the next large goal into `playback-streaming` if we continue improving
remote playback internals. Otherwise, continue with remote-storage hardening:
disk budget/cleanup, remote body streaming, and multi-library configuration.
