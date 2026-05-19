# Android Presentation Runtime Adapters - TODO

Status: Closed
Last updated: 2026-05-20

## M0 - Scope And Evidence Freeze

- [x] APRA-010 [owner=codex] [deps=none] [scope=docs/workstreams/android-presentation-runtime-adapters]
  Goal: Freeze problem, target state, non-goals, and evidence anchors.
  Validation: DESIGN.md, MILESTONES.md, EVIDENCE_AND_GATES.md, WORKSTREAM.json, and HANDOFF.md exist and agree.
  Evidence: docs/workstreams/android-presentation-runtime-adapters/DESIGN.md
  Handoff: DONE. Lane docs created and aligned with the player session follow-on.

## M1 - Artwork Runtime Adapter

- [x] APRA-020 [owner=codex] [deps=APRA-010] [scope=apps/android/app/src/main/java/dev/taru/android/ui/artwork,apps/android/app/src/main/java/dev/taru/android/ui/browse]
  Goal: Move token-backed artwork request creation behind a Browse runtime adapter and remove shell-local token reads for Home and Libraries artwork.
  Validation: apps\android\gradlew.bat -p apps\android :app:testDebugUnitTest --tests dev.taru.android.ui.artwork.* --tests dev.taru.android.ui.browse.* --no-daemon --no-parallel
  Review: Confirm visual screens receive presentation-safe artwork inputs or a small resolver only.
  Evidence: focused JVM test output and git diff.
  Handoff: DONE. Added `ArtworkRequestResolver` and token-vault production adapter; Browse shell no longer reads tokens for Home/Libraries artwork.

## M2 - Detail Presentation Contract

- [x] APRA-030 [owner=codex] [deps=APRA-020] [scope=apps/android/app/src/main/java/dev/taru/android/ui/screens/detail,apps/android/app/src/main/java/dev/taru/android/ui/browse]
  Goal: Remove `ServerProfile` and raw `accessToken` from detail visual APIs by passing an artwork request resolver/presentation model.
  Validation: apps\android\gradlew.bat -p apps\android :app:testDebugUnitTest --tests dev.taru.android.ui.browse.BrowseSessionLoadingTest --tests dev.taru.android.ui.artwork.* --no-daemon --no-parallel
  Review: Detail should remain a pure rendering surface plus callbacks.
  Evidence: focused JVM test output and API diff.
  Handoff: DONE. Detail route now receives `ArtworkRequestResolver`; source picker, playback decision, and playback callbacks are unchanged.

## M3 - Player Route Runtime Renderer

- [x] APRA-040 [owner=codex] [deps=APRA-030] [scope=apps/android/app/src/main/java/dev/taru/android/ui/browse,apps/android/app/src/main/java/dev/taru/android/ui/screens/player]
  Goal: Add a narrow player route renderer/runtime adapter so Browse shell renders a `PlaybackLaunchRequest` without knowing player internals.
  Validation: apps\android\gradlew.bat -p apps\android :app:testDebugUnitTest --tests dev.taru.android.ui.browse.* --tests dev.taru.android.player.* --no-daemon --no-parallel
  Review: Do not rewrite ExoPlayer lifecycle or exit effects in this task.
  Evidence: focused JVM test output and git diff.
  Handoff: DONE. Browse shell now renders Player route through `PlayerRouteRenderer`; deeper Media3 lifecycle ownership remains in `android-player-session-architecture`.

## M4 - Closeout

- [x] APRA-050 [owner=codex] [deps=APRA-040] [scope=docs/workstreams/android-presentation-runtime-adapters]
  Goal: Verify final gates, close the lane, and record follow-ons.
  Validation: apps\android\gradlew.bat -p apps\android :app:testDebugUnitTest --no-daemon --no-parallel; git diff --check
  Review: No raw access token props in Browse/detail presentation APIs; no shell-local artwork token reads.
  Evidence: EVIDENCE_AND_GATES.md, WORKSTREAM.json
  Handoff: DONE. Final unit test and diff checks passed; player session architecture remains the next lane.
