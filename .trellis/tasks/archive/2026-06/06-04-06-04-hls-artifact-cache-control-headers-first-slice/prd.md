# HLS artifact cache-control headers first slice

## Goal

Make HLS playlist and segment HTTP responses carry explicit conservative cache
headers so session-scoped playback artifacts are not accidentally stored or
reused by browsers, proxies, or shared caches before Nako has a broader
artifact cache contract.

## What I Already Know

* `docs/architecture/CONTROL_PLANE.md` lists HTTP cache/ETag contracts as a
  narrow partial and calls out artifact cache semantics as a follow-on.
* `crates/nako-server/src/http/playback.rs` owns HLS playlist and segment route
  responses.
* `hls_playlist_response` currently sets content type/content length and
  playback session id but no cache policy.
* HLS segment responses currently reuse direct-play byte response headers
  through `stream_local_file_response` / `apply_direct_play_headers`, which
  does not add a cache policy.
* HLS playlist and segment URLs are session/ticket scoped and can contain
  sensitive playback authorization context even when response bodies are safe.

## Assumptions

* The first slice should be conservative: use `Cache-Control: no-store` for HLS
  playlist and segment responses.
* No-store is intentionally less cache-efficient than immutable segment caching,
  but it avoids access-control mistakes until a future ETag/immutable-artifact
  contract is designed.
* Direct Play and Remux response cache behavior should remain unchanged in this
  slice.

## Requirements

* Add explicit HLS artifact cache-control behavior in `nako-server` HTTP
  playback responses.
* HLS playlist responses must include `Cache-Control: no-store`.
* HLS segment responses must include `Cache-Control: no-store`.
* Preserve existing content type, content length, range, playback session id,
  and status behavior.
* Keep the helper focused on HLS artifact responses; do not change Direct Play
  or Remux headers.
* Add focused HTTP tests for playlist and segment cache headers.

## Acceptance Criteria

* [x] HLS playlist route responses include `Cache-Control: no-store`.
* [x] HLS segment route responses include `Cache-Control: no-store`.
* [x] Existing HLS playlist/segment route tests still pass.
* [x] No public/Admin DTO, generated contract, schema, playback planner, or
  transcode command planning changes are made.
* [x] `cargo fmt --all -- --check`, `cargo check -p nako-server --tests`,
  focused `cargo nextest`, and `git diff --check` pass.

## Definition Of Done

* Code and tests are committed with a Conventional Commit message.
* Task evidence records verification commands.
* Relevant Trellis spec and architecture docs are updated with the HLS
  no-store artifact cache convention.
* The task is archived and the developer journal is recorded.

## Out Of Scope

* No ETag, Last-Modified, conditional GET, or immutable segment policy.
* No image/artwork/catalog cache semantics.
* No Direct Play, Remux, subtitle, addon asset, or Admin diagnostics cache
  behavior changes.
* No public DTO/generated contract/schema changes.
* No CDN/proxy configuration docs beyond the architecture/spec note.

## Technical Approach

Add a small HLS-only response helper in `http/playback.rs` that inserts
`Cache-Control: no-store`. Call it from `hls_playlist_response` and after HLS
segment file response planning. Update the existing
`hls_playlist_and_segment_routes_work` test or add a focused companion test to
assert both headers while preserving current content and session behavior.

## Research References

* `research/current-hls-cache-header-state.md`

## Technical Notes

Likely specs:

* `.trellis/spec/nako-server/backend/index.md`
* `.trellis/spec/nako-server/backend/http-api-patterns.md`
* `.trellis/spec/nako-server/backend/quality-guidelines.md`
* `.trellis/spec/guides/cross-layer-thinking-guide.md`

Likely write scope:

* `crates/nako-server/src/http/playback.rs`
* `crates/nako-server/src/http/tests/playback.rs`
* `.trellis/spec/nako-server/backend/http-api-patterns.md`
* `docs/architecture/CONTROL_PLANE.md`

## Verification

* PASS: `cargo fmt --all`
* PASS: `cargo check -p nako-server --tests`
* PASS:
  `cargo nextest run -p nako-server hls_playlist_and_segment_routes_work --no-fail-fast`
* PASS: `cargo nextest run -p nako-server hls_playlist --no-fail-fast`

* PASS: `cargo fmt --all -- --check`
* PASS:
  `python .\\.trellis\\scripts\\task.py validate 06-04-06-04-hls-artifact-cache-control-headers-first-slice`
* PASS: `git diff --check`

## Spec Update

* Updated `.trellis/spec/nako-server/backend/http-api-patterns.md` with the
  HLS artifact `Cache-Control: no-store` convention.
* Updated `docs/architecture/CONTROL_PLANE.md` to record the HLS no-store
  baseline while keeping immutable/ETag/cache-key semantics as follow-ons.
