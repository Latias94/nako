# HLS Subtitle Route Regression Fix

## Goal

Fix the deterministic full-workspace regression where existing HLS subtitle
HTTP routes fail with `hls subtitle burn-in supports only embedded subtitle
streams` after the subtitle burn-in planning slice.

## Known Failing Tests

* `cargo nextest run -p nako-server http::tests::playback::hls_playlist_route_accepts_preferred_subtitle_language_defaults --no-fail-fast`
* `cargo nextest run -p nako-server http::tests::playback::hls_playlist_route_subtitle_stream_overrides_preferred_subtitle_language --no-fail-fast`

## Requirements

* Preserve the new playback/transcode burn-in planning behavior for ASS/SSA and
  remux blocking.
* Restore existing HLS sidecar subtitle route behavior for supported subtitle
  selections.
* Keep the fix minimal and focused on the mismatch between server subtitle
  source facts and the new transcode burn-in/sidecar contract.
* Do not change public API shape, schema, generated contracts, or Admin Web.

## Acceptance Criteria

* [x] Both known failing HLS subtitle route tests pass.
* [x] Focused playback/transcode tests from the burn-in slice still pass.
* [x] Full workspace nextest passes or any remaining failures are unrelated and
      documented.
* [x] `cargo fmt --all -- --check` and `git diff --check` pass.

## Out Of Scope

* New subtitle capability API.
* PGS/image subtitle burn-in execution.
* HLS seek/restart changes.
* Broad server playback refactor.

## Evidence

Root cause: the burn-in planning slice treated `webvtt` as sidecar-capable but
missed the existing `vtt` codec/extension alias used by server sidecar subtitle
probe facts. That made supported sidecar subtitles select `BurnInSelected`,
which the transcode adapter correctly rejected for external subtitle origins.

Verification:

* `cargo nextest run -p nako-playback sidecar_capable_hls_subtitle_format_remains_sidecar_selected --no-fail-fast`
* `cargo nextest run -p nako-server http::tests::playback::hls_playlist_route_accepts_preferred_subtitle_language_defaults http::tests::playback::hls_playlist_route_subtitle_stream_overrides_preferred_subtitle_language --no-fail-fast`
* `cargo check -p nako-playback -p nako-transcode -p nako-server --tests`
* `cargo nextest run -p nako-playback --no-fail-fast`
* `cargo nextest run -p nako-transcode hls --no-fail-fast`
* `cargo nextest run --workspace --no-fail-fast`
* `cargo fmt --all -- --check`
* `git diff --check`
