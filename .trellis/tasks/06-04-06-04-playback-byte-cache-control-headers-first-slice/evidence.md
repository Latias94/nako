# Evidence: playback byte cache-control headers first slice

## Implementation Summary

Direct Play and Remux media byte responses now include
`Cache-Control: no-store` through the shared `apply_direct_play_headers`
helper. This covers GET/range streaming responses and HEAD/preflight empty
responses while preserving existing status, range headers, content headers,
playback session headers, auth/ticket behavior, and bodies.

## Files Changed

* `crates/nako-server/src/http/playback.rs`
* `crates/nako-server/src/http/tests/playback.rs`
* `.trellis/spec/nako-server/backend/http-api-patterns.md`
* `docs/architecture/CONTROL_PLANE.md`
* `docs/architecture/PLAYBACK.md`
* `.trellis/tasks/06-04-06-04-playback-byte-cache-control-headers-first-slice/prd.md`
* `.trellis/tasks/06-04-06-04-playback-byte-cache-control-headers-first-slice/research/playback-byte-cache-control.md`

## Verification

* PASS: `cargo fmt --all -- --check`
* PASS: `cargo check -p nako-server --tests`
* PASS: `cargo nextest run -p nako-server direct_stream_head_returns_headers_without_body --no-fail-fast`
* PASS: `cargo nextest run -p nako-server direct_stream_route_records_playback_session_without_transcode_artifact --no-fail-fast`
* PASS: `cargo nextest run -p nako-server remux_stream_route_runs_and_reuses_completed_output --no-fail-fast`
* PASS: `cargo nextest run -p nako-server head_remux_stream_route_exposes_session_without_body --no-fail-fast`
* PASS: `git diff --check`
* PASS: `python ./.trellis/scripts/task.py validate .trellis/tasks/06-04-06-04-playback-byte-cache-control-headers-first-slice`

## Notes

* HLS remains on its dedicated `apply_hls_artifact_cache_headers` helper.
* Selected artwork remains on its private cache-control and ETag/304 contract.
* Direct Play/Remux ETags, conditional GET, immutable/shared-cache behavior,
  DTO/generated-contract changes, and schema changes remain out of scope.
