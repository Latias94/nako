# Phase 6.3: Remote Probe Staging

## Status

Completed.

## Objective

Allow media probing to inspect WebDAV sources through deterministic local
staging when a storage backend cannot provide a local path hint.

## Implemented

- Added VFS staging contract:
  - `StageRequest`;
  - `StagedFile`;
  - default unsupported `StorageBackend::stage`;
  - deterministic staging path helper keyed by source URI and
    etag/fingerprint.
- Added WebDAV staging support:
  - validates the source is a file;
  - reuses an existing staged file when size still matches;
  - downloads with WebDAV GET into a temporary file;
  - promotes the completed file into the deterministic staging path;
  - keeps credentials out of staged file names and locators.
- Added local backend staging support that reuses the local path hint.
- Updated `LibraryProbeService`:
  - uses `VirtualFile.local_path_hint` when available;
  - falls back to `StorageBackend::stage` when a `staging_root` is configured;
  - passes the staged local path to `MediaProbe`.

## Validation

- `cargo test -p nako-vfs webdav_backend_stages_file_to_deterministic_local_path`
- `cargo test -p nako-library probe_service_stages_webdav_source_before_probe`

## Boundary Notes

- Staging is a VFS concern, not a WebDAV-specific probe behavior.
- `FfprobeMediaProbe` still requires a local path; M6.3 supplies that path
  before calling probe.
- Staging paths are deterministic and do not contain source credentials.
- This phase validates reuse by size. Etag/fingerprint participates in the
  deterministic path key, so a changed remote fingerprint maps to a different
  staged path.

## Remaining Gaps

- No persistent staging manifest yet.
- No disk budget or cleanup worker yet.
- Server config uses `remux_staging_root/probe-inputs` for remote probe staging;
  there is no independent probe staging root yet.
- Remux and HLS staging remain M6.4 work.

## Next Step

Proceed to M6.4 remote playback policy: remote direct range streaming where
possible, and explicit remux/HLS staging behavior for FFmpeg inputs.
