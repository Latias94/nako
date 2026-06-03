# Current HLS Cache Header State

## Current Code Shape

`crates/nako-server/src/http/playback.rs` owns the HLS HTTP response boundary.

Relevant functions:

* `hls_playlist_source` returns HLS playlist responses.
* `hls_segment` returns HLS segment responses.
* `hls_playlist_response` sets playlist content type, content length, and
  optional playback session id.
* `stream_local_file_response` and `apply_direct_play_headers` set byte
  streaming headers for local file responses, including HLS segment files.

Current HLS playlist and segment responses do not set `Cache-Control`.

## Architecture Constraint

`docs/architecture/CONTROL_PLANE.md` says HTTP cache/ETag contracts are only a
narrow shipped partial and still need systematic image, artifact, and catalog
semantics. Its risk register notes that cache correctness is access control.

HLS playlist and segment URLs are session-scoped. Browser playback tickets and
renderer tickets may authorize access through query parameters, while HLS
playlist bodies author session route URLs. Until Nako has a broader token-aware
cache key and immutable artifact model, shared caching should be explicitly
disabled for these responses.

## Recommended First Slice

Use `Cache-Control: no-store` for:

* HLS playlist responses from `hls_playlist_response`.
* HLS segment responses from `hls_segment` after local file response creation.

Do not alter:

* Direct Play stream responses.
* Remux stream responses.
* Subtitle responses.
* Artwork/cache/ETag behavior.

## Test Targets

Update `http::tests::playback::hls_playlist_and_segment_routes_work` because it
already exercises a real HLS playlist response, segment route response, session
header, segment content type, and segment body.

Assertions:

* playlist response header `Cache-Control` is exactly `no-store`;
* segment response header `Cache-Control` is exactly `no-store`;
* existing content type/session/body assertions continue to pass.

## Risks

* Adding `Cache-Control` through `apply_direct_play_headers` would change Direct
  Play and Remux behavior, which is out of scope.
* Using a future-looking immutable segment policy now could create access
  control bugs because session/ticket cache keys are not yet documented or
  tested.
* Adding ETag/Last-Modified would require a broader artifact identity and
  conditional response contract.
