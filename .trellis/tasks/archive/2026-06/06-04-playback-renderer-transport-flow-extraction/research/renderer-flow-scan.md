# Renderer Flow Scan

## Summary

`PlaybackAppService::start_renderer_playback_session` is still broad
app-service orchestration. It can be extracted into a focused
`app/playback/renderer_flow.rs` module without changing HTTP renderer routes or
public API shape.

## Current Entry Point

Current code in `crates/nako-server/src/app/playback/mod.rs`:

* `StartRendererPlaybackSessionRequest`
* `StartRendererPlaybackSessionOutput`
* `RendererPlaybackTransportPlan`
* `PlaybackAppService::start_renderer_playback_session`

The method currently owns:

* source lookup;
* media probe lookup;
* playback selection context;
* effective policy resolution;
* `RemoteControl` permission enforcement;
* playback planning;
* Direct playback session creation;
* Remux startup through `start_remux_source_with_policy`;
* Remux playback-session creation and transcode linkage;
* HLS startup through `hls_playlist_with_policy`;
* HLS playback-session creation, transcode linkage, and supersede cleanup;
* renderer transport plan construction.

## Extraction Target

Add `crates/nako-server/src/app/playback/renderer_flow.rs` and move the method
body there:

```rust
renderer_flow::start_renderer_playback_session(self, request).await
```

## Boundary Constraints

* Keep renderer HTTP ticket/url authoring in `http/renderer.rs`.
* Keep playback compatibility rules in `nako-playback`.
* Keep Remux startup details in `remux_flow`.
* Keep HLS startup details in `hls_flow`.
* Do not introduce API/DTO/schema changes.

## Likely Helper Reuse

The renderer flow should reuse existing app helpers and sibling flow functions:

* `get_source_or_not_found`
* `playback_selection_context_for_source`
* `effective_playback_policy_for_source`
* `start_playback_session`
* `link_playback_session_transcode`
* `cancel_superseded_hls_playback_sessions`
* `remux_flow::start_remux_source_with_policy`
* `hls_flow::hls_playlist_with_policy`
* `selection::remux_output_container`

## Suggested Verification

* `cargo fmt --all -- --check`
* `cargo check -p nako-server --tests`
* `cargo nextest run -p nako-server renderer --no-fail-fast`
* `cargo nextest run -p nako-server remux --no-fail-fast` if Remux helper
  visibility or call paths change
* `cargo nextest run -p nako-server hls_playlist --no-fail-fast` if HLS helper
  visibility or call paths change
* `git diff --check`
