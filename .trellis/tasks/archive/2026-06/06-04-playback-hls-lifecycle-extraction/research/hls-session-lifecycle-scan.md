# HLS Session Lifecycle Extraction Scan

## Summary

The HLS source and playlist startup boundary already exists in
`crates/nako-server/src/app/playback/hls_flow.rs`. The remaining extraction
candidate is the playback-session-facing playlist orchestration that still
lives in `crates/nako-server/src/app/playback/mod.rs`.

## Existing Boundaries

* `app/playback/hls_flow.rs`
  * owns `hls_source_with_policy`;
  * owns `hls_playlist_with_policy`;
  * builds HLS source context;
  * plans HLS runtime requests through typed playback/transcode boundaries;
  * performs `HlsStart` and `HlsSupersede` admission;
  * starts background HLS work through the runtime supervisor;
  * waits for playlist readiness or terminal HLS session states.
* `app/playback/hls.rs`
  * owns reserved HLS runner execution;
  * persists transcode session state around FFmpeg;
  * owns in-flight HLS request admission for the runner.
* `app/playback/mod.rs`
  * still owns `hls_playlist_playback`;
  * still owns `hls_playlist_for_playback_session`;
  * starts or validates playback sessions;
  * links playback sessions to HLS transcode sessions;
  * cancels superseded HLS playback sessions;
  * reads playback-decorated HLS playlists from `hls_artifacts`.

## Extraction Target

Move the following `mod.rs` HLS session-facing entrypoints into `hls_flow.rs`:

* `hls_playlist_playback`
* `hls_playlist_for_playback_session`

Keep the public `PlaybackAppService` methods as thin delegators:

```rust
hls_flow::hls_playlist_playback(self, request).await
hls_flow::hls_playlist_for_playback_session(self, request).await
```

## Expected Helper Visibility

The extraction can reuse existing `PlaybackAppService` helpers instead of
duplicating logic:

* `effective_playback_policy_for_source_id`
* `start_playback_session`
* `link_playback_session_transcode`
* `cancel_superseded_hls_playback_sessions`
* `existing_playback_session_for_media_request`
* `client_capabilities_for_playback_session`
* `get_transcode_session`

If needed, adjust helper visibility narrowly to `pub(super)`.

## Constraints

* Do not change public API, DTO, route, schema, or generated SDK shape.
* Do not move FFmpeg command planning or playback compatibility decisions into
  server flow code.
* Preserve `HlsStart` for ordinary startup and `HlsSupersede` for replacement
  flows.
* Preserve trace request ID propagation into HLS playlist startup.
* Preserve manifest-backed playlist and segment URL authority.

## Suggested Focused Verification

* `cargo fmt --all -- --check`
* `cargo check -p nako-server --tests`
* `cargo nextest run -p nako-server hls_playlist --no-fail-fast`
* `cargo nextest run -p nako-server hls_source --no-fail-fast` if the source
  context path changes unexpectedly
* `git diff --check`
