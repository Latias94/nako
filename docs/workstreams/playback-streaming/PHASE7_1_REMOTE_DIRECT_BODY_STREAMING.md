# Phase 7.1: Remote Direct Body Streaming

Status: active; body streaming foundation implemented.

## Goal

Remove the highest-risk remote direct-play behavior: buffering selected WebDAV
range bytes into an in-memory `Vec<u8>` before returning the HTTP response.

## Implemented Shape

- Added `taru_vfs::ReadStream` and `StorageBackend::stream_range`.
- `WebDavBackend::stream_range` now proxies the `reqwest` byte stream instead
  of accumulating chunks into memory.
- Direct play returns `DirectPlaySourceBody::Stream` for remote sources without
  local path hints.
- `GET /sources/{source_id}/stream` translates remote VFS streams directly
  into axum response bodies while preserving direct-play headers.
- `HEAD /sources/{source_id}/stream` uses a preflight plan and no longer opens
  the direct-play body.
- Playback app planning moved to `crates/taru-server/src/app/playback.rs`.
- Playback HTTP response helpers moved to
  `crates/taru-server/src/http/playback.rs`.

## Validation

Focused validation:

- `cargo check -p taru-vfs -p taru-server`
- `cargo nextest run -p taru-vfs webdav_backend_streams_byte_ranges_with_http_range_header`
- `cargo nextest run -p taru-server direct_play_uses_vfs_stream_when_backend_has_no_local_path direct_stream_response_proxies_vfs_body_stream direct_stream_head_returns_headers_without_body`

## Remaining Gaps

- `ReadRange` remains available for small in-process reads and legacy tests,
  but direct play no longer uses it for remote response bodies.
- `playback.remote.stream` budget acquisition is still M7.4 work.
- Staging manifest, disk budget, cleanup, and richer playback error taxonomy
  remain M7.2 and M7.3 work.
- This phase starts server module splitting; broader `taru-db` and route-module
  splits remain follow-up refactors before the staging manifest grows the large
  files further.
