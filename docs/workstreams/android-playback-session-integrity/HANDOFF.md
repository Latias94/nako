# Android Playback Session Integrity - Handoff

Status: Closed
Last updated: 2026-05-19

## Current State

The lane is active. Direct Play smoke is already proven by
`android-playback-depth-validation`; this lane starts where that one deferred
HLS/remux/session depth.

APSI-020, APSI-030, and APSI-040 are complete. The Public Client API now exposes
`x-taru-playback-session-id` for remux/HLS session-backed playback responses,
the Rust and TypeScript clients expose remux `HEAD` preflight support, and
Android carries the observed session id into `PlaybackLaunchRequest`. The
`profile-with-media` smoke fixture now also writes a token-safe public remux
session readback artifact. Player exit side effects are extracted into a
test-covered collaborator that requests session cancellation for unfinished
playback with a non-blank session id.

## Current Task

None. This workstream is closed.

## Key Constraints

- Do not parse HLS playlist text in Android just to discover a session id.
- Do not use admin-only playback diagnostics as Android client authority.
- Do not expose bearer tokens, local filesystem paths, or provider command
  lines in smoke artifacts.
- Direct Play remains sessionless.

## Useful Context

- `apps/android/app/src/main/java/dev/taru/android/ui/browse/TaruBrowseShell.kt`
  now launches the player with `target.sessionId`.
- `apps/android/app/src/main/java/dev/taru/android/playback/TaruPlaybackClient.kt`
  prepares remux/HLS targets by reading the public playback session header.
- `crates/taru-server/src/http/playback.rs` exposes session inspection and
  cancellation routes, while remux/HLS stream routes expose session identity
  through `x-taru-playback-session-id`.
- `apps/android/scripts/Smoke-Emulator.ps1` writes
  `profile-with-media-session-readback.txt` by creating a remux session through
  Public Client API `HEAD` preflight and reading it back after the Android
  player returns to detail.
- `apps/android/app/src/main/java/dev/taru/android/player/PlaybackExitEffects.kt`
  owns player-exit persistence, User Playback State reporting, and session
  cancellation semantics.

## Verified Evidence

- `cargo test -p taru-client-protocol -p taru-api -p taru-client --lib`
- `cargo test -p taru-server remux_stream_route -- --nocapture`
- `cargo test -p taru-server hls_playlist_and_segment_routes_work -- --nocapture`
- `apps\android\gradlew.bat -p apps\android :app:testDebugUnitTest --tests dev.taru.android.playback.TaruPlaybackClientTest --no-daemon`
- `apps\android\gradlew.bat -p apps\android :app:compileDebugKotlin --no-daemon`
- `npm run check --prefix sdk/typescript`
- `cargo fmt --check`
- `git diff --check`
- `pwsh -NoProfile -Command "[scriptblock]::Create((Get-Content -LiteralPath 'apps/android/scripts/Smoke-Emulator.ps1' -Raw)) | Out-Null; 'Smoke-Emulator parse ok'"`
- `pwsh -NoProfile -File apps/android/scripts/Smoke-Emulator.ps1 -FixtureState profile-with-media`
  produced
  `apps/android/build/smoke/20260519-190847-profile-with-media-emulator-5554/profile-with-media-session-readback.txt`.
- `apps\android\gradlew.bat -p apps\android :app:testDebugUnitTest --tests dev.taru.android.player.PlaybackExitEffectsTest --tests dev.taru.android.player.UserPlaybackReportingTest --tests dev.taru.android.playback.TaruPlaybackClientTest --no-daemon`
- `apps\android\gradlew.bat -p apps\android :app:compileDebugKotlin --no-daemon`

## Residual Follow-On

Add a deliberate long-media or non-Direct runtime fixture that forces the
Android player to consume remux/HLS and exit before completion. Use that to
prove active session cancellation through `/playback/sessions/{session_id}`.
The current `profile-with-media` UI path remains Direct Play and is not valid
evidence for active remux/HLS player cancellation.
