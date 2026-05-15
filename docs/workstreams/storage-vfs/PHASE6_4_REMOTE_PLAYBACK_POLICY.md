# Phase 6.4: Remote Playback Policy

## Status

Completed.

## Objective

Make playback choose explicit remote-safe paths when a source does not expose a
backend-local file path.

## Implemented

- Added VFS byte-range read support:
  - `ReadRange`;
  - default unsupported `StorageBackend::read_range`;
  - local backend range reads;
  - cached backend delegation;
  - WebDAV HTTP `Range` GET support.
- Updated direct play planning:
  - local sources continue streaming from local paths;
  - range-readable remote sources without local path hints are read through
    VFS and returned as response bytes;
  - invalid or unsatisfiable ranges return the existing empty 416 response
    path.
- Updated remux and HLS input selection:
  - local path hints are reused when available;
  - remote sources without local path hints are staged under
    `remux_staging_root/inputs` before FFmpeg planning;
  - source locators and credentials are not passed to FFmpeg as input paths.
- Added range validation for zero-length VFS ranges.

## Validation

- `cargo test -p taru-vfs webdav_backend_reads_byte_ranges_with_http_range_header`
- `cargo test -p taru-vfs webdav_backend_stages_file_to_deterministic_local_path`
- `cargo test -p taru-server direct_play_uses_vfs_bytes_when_backend_has_no_local_path`
- `cargo test -p taru-server ffmpeg_source_path_stages_remote_backend_without_local_path_hint`
- `cargo test -p taru-server ffmpeg_source_path_reuses_local_path_hint_without_staging`

## Boundary Notes

- M6.4 proves playback policy at the VFS/app boundary. Production server
  configuration is wired in M6.5; M6.4's focus remains the playback policy.
- Direct play currently materializes the selected remote range into memory.
  This is acceptable for the policy preview, but streaming remote response
  bodies should replace it before large remote direct-play usage.
- Remux and HLS stage full remote objects because FFmpeg receives local file
  paths, not remote URLs or credentials.
- Playback decision scoring still needs richer storage capability awareness.

## Remaining Gaps

- No remote direct-play body streaming yet; range bytes are buffered in memory.
- No disk budget, cleanup worker, or persistent staging manifest yet.
- Remote storage timeout and stale-cache failure mapping are not fully surfaced
  through playback responses.

## Next Step

Proceed to M6.5 stabilization: wire preview configuration/docs/API notes,
document known limitations, run full workspace validation, and decide whether
the next large goal should be remote configuration hardening or a dedicated
playback-streaming workstream.
