# Android Device-Local Playback Position - TODO

Status: Closed
Last updated: 2026-05-19

## M0 - Boundary Freeze

- [x] ADP-010 [owner=planner] [deps=none] [scope=docs/workstreams/android-device-local-playback-position]
  Goal: Open the lane and freeze the boundary between device-local persistence
  and server-authoritative **User Playback State**.
  Validation: Workstream docs exist and agree.
  Evidence: `docs/workstreams/android-device-local-playback-position/DESIGN.md`
  Handoff: Completed on 2026-05-19. First implementation slice is an Android
  SharedPreferences-backed `DevicePlaybackPositionStore`.

## M1 - Persistent Device-Local Store

- [x] ADP-020 [owner=codex] [deps=ADP-010] [scope=apps/android/app/src/main/java/dev/nako/android/player,apps/android/app/src/main/java/dev/nako/android/ui,apps/android/app/src/test/java/dev/nako/android/player]
  Goal: Implement a persistent Android `DevicePlaybackPositionStore`, wire it
  into app composition, and keep the in-memory store for previews/tests.
  Validation:
  `apps\android\gradlew.bat -p apps\android :app:testDebugUnitTest --tests dev.nako.android.player.PlaybackLaunchTest --no-daemon`
  plus `pwsh -NoProfile -File apps\android\scripts\Validate-AndroidLocal.ps1 -SkipSmoke`
  and `git diff --check`.
  Review: Check that stored data is local-only, scoped by profile/item/source,
  and does not alter Public Client API behavior.
  Evidence: `EVIDENCE_AND_GATES.md`.
  Handoff: DONE on 2026-05-19. Added
  `SharedPreferencesDevicePlaybackPositionStore`, wired it into
  `NakoAndroidApp`, and covered persistence across store instances, scoped
  lookup, non-positive clearing, and corrupt local data fallback.

## M2 - Closeout

- [x] ADP-030 [owner=planner] [deps=ADP-020] [scope=docs/workstreams/android-device-local-playback-position]
  Goal: Verify evidence, close this local persistence lane, and keep
  cross-device resume/session-envelope work split.
  Validation: fresh ADP-020 gates and closeout doc updates.
  Review: Use review-workstream and verify-rust-workstream before closeout.
  Evidence: `EVIDENCE_AND_GATES.md`, `WORKSTREAM.json`, `HANDOFF.md`.
  Handoff: DONE on 2026-05-19. Server-authoritative **User Playback State**,
  cross-device Continue Watching, progress reporting, and playback stream
  session-envelope work remain separate follow-ons.
