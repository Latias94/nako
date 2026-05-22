# Phase 7.1: Remote Direct Body Streaming

Status: completed for M7.

## Goal

Remove the highest-risk remote direct-play behavior: buffering selected WebDAV
range bytes into an in-memory `Vec<u8>` before returning the HTTP response.

## Implemented Shape

- Added `nako_vfs::ReadStream` and `StorageBackend::stream_range`.
- `WebDavBackend::stream_range` now proxies the `reqwest` byte stream instead
  of accumulating chunks into memory.
- Direct play returns `DirectPlaySourceBody::Stream` for remote sources without
  local path hints.
- `GET /sources/{source_id}/stream` translates remote VFS streams directly
  into axum response bodies while preserving direct-play headers.
- `HEAD /sources/{source_id}/stream` uses a preflight plan and no longer opens
  the direct-play body.
- Playback app planning moved to `crates/nako-server/src/app/playback.rs`.
- Playback HTTP response helpers moved to
  `crates/nako-server/src/http/playback.rs`.

## Validation

Focused validation:

- `cargo check -p nako-vfs -p nako-server`
- `cargo nextest run -p nako-vfs webdav_backend_streams_byte_ranges_with_http_range_header`
- `cargo nextest run -p nako-server direct_play_uses_vfs_stream_when_backend_has_no_local_path direct_stream_response_proxies_vfs_body_stream direct_stream_head_returns_headers_without_body`

## Remaining Gaps

- `ReadRange` remains available for small in-process reads and legacy tests,
  but direct play no longer uses it for remote response bodies.
- `ReadRange` remains available for small in-process reads and tests, but
  direct play no longer uses it for remote response bodies.
- Broader `nako-db` and route-module splits remain follow-up refactors outside
  the M7 direct body streaming exit criteria.
