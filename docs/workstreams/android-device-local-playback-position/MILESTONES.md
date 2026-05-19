# Android Device-Local Playback Position - Milestones

Status: Closed
Last updated: 2026-05-19

## M0 - Boundary Freeze

Exit criteria:

- Device-local persistence is clearly separated from **User Playback State**.
- First implementation slice and non-goals are explicit.

Status: Complete.

## M1 - Persistent Device-Local Store

Exit criteria:

- A SharedPreferences-backed store implements `DevicePlaybackPositionStore`.
- App composition uses the persistent store.
- In-memory store remains available for previews/tests.
- Tests prove persistence across store instances, key scoping, clearing, and
  corrupt local data fallback.

Primary gates:

- `apps\android\gradlew.bat -p apps\android :app:testDebugUnitTest --tests dev.taru.android.player.PlaybackLaunchTest --no-daemon`
- `pwsh -NoProfile -File apps\android\scripts\Validate-AndroidLocal.ps1 -SkipSmoke`
- `git diff --check`

Primary evidence:

- `apps\android\gradlew.bat -p apps\android :app:testDebugUnitTest --tests dev.taru.android.player.PlaybackLaunchTest --no-daemon`
- `apps/android/build/validation/20260519-100247/report.md`
- `git diff --check`

Status: Complete.

## M2 - Closeout

Exit criteria:

- Evidence is recorded.
- `WORKSTREAM.json` status is updated.
- Follow-ons for server-authoritative playback state and playback session
  launch envelopes remain split.

Status: Complete.
