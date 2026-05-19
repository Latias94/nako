# Android Active Playback Session Cancellation - Evidence And Gates

Status: Closed
Last updated: 2026-05-19

## Required Gates

- Focused server HTTP test for active remux preflight/start and cancellation.
- Focused Android unit tests for debug fixture playback capabilities and
  playback decision capability query behavior.
- Script parse for changed PowerShell scripts.
- Focused smoke command for the active-remux state.
- `cargo fmt --check` or narrower formatting evidence if the workspace is too
  large for the turn.
- `git diff --check`.

## Evidence Log

- 2026-05-19: Lane opened.
- 2026-05-19: `cargo test -p taru-transcode remux_runner_kills_and_cleans_temp_output_on_cancel -- --nocapture` passed. Proves remux cancellation does not wait on inherited stderr pipes before returning a cancelled outcome.
- 2026-05-19: `cargo test -p taru-server remux_stream_route -- --nocapture` passed. Proves Public Client remux HEAD preflight exposes an active session id, duplicate remux stream reuse, and finished-output reuse.
- 2026-05-19: `apps\android\gradlew.bat -p apps\android :app:testDebugUnitTest --tests dev.taru.android.playback.TaruPlaybackClientTest --tests dev.taru.android.player.PlaybackExitEffectsTest --tests dev.taru.android.playback.PlaybackPreferencesStoreTest --no-daemon --no-parallel` passed. Proves Android preserves forced playback capabilities through decision/start target construction and requests session cancellation before progress sync on player exit.
- 2026-05-19: PowerShell parse passed for `Smoke-Emulator.ps1`, `Smoke-Regression.ps1`, and `Start-DemoFixtureServer.ps1`.
- 2026-05-19: `pwsh -NoProfile -File apps/android/scripts/Smoke-Emulator.ps1 -FixtureState profile-active-remux -FixtureServerPort 3033` passed. Evidence directory: `apps/android/build/smoke/20260519-223623-profile-active-remux-emulator-5554/`. Public cancellation readback artifact: `profile-active-remux-session-cancelled.txt`, observed `kind=remux`, `state=cancelled`, and `failure_category=cancelled` through `/playback/sessions/{session_id}`.

## Closeout Notes

- Android source picker now checks playback compatibility without starting a session; session preflight moves to the Start playback action.
- The debug-only `profile-active-remux` fixture forces an MKV source to choose Remux while normal product playback capabilities stay unchanged.
- Generated smoke artifacts are not committed.
