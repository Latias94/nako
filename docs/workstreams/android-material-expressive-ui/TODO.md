# Android Material Expressive UI — TODO

Status: Completed
Last updated: 2026-05-18

## M0 — Scope And Evidence Freeze

- [x] AME-010 [owner=planner] [deps=none] [scope=docs/workstreams/android-material-expressive-ui]
  Goal: Open the durable Android UI rewrite lane and freeze V2 design,
  Material 3 Expressive, dynamic color, animation, and implementation
  boundaries.
  Validation: Workstream docs exist and agree.
  Evidence: `docs/workstreams/android-material-expressive-ui/DESIGN.md`
  Handoff: Completed before code rewrite starts.

## M1 — UI Foundation Rewrite

- [x] AME-020 [owner=codex] [deps=AME-010] [scope=apps/android/app/src/main/java/dev/nako/android/ui/theme,apps/android/app/src/main/java/dev/nako/android/ui/components,apps/android/app/src/main/java/dev/nako/android/ui/shell]
  Goal: Replace the tracer-era UI foundation with a Material 3 Expressive-ready
  design-system layer: dark-first color roles, optional dynamic color,
  artwork-accent hooks, motion tokens, adaptive phone/tablet chrome, and shared
  state surfaces.
  Validation: `apps\android\gradlew.bat -p apps\android :app:testDebugUnitTest --no-daemon`; `apps\android\gradlew.bat -p apps\android :app:assembleDebug --no-daemon`; `git diff --check`
  Review: Use review-workstream before accepting completion.
  Evidence: `apps/android/app/src/main/java/dev/nako/android/ui/theme/Theme.kt`, `apps/android/app/src/main/java/dev/nako/android/ui/theme/ArtworkAccent.kt`, `apps/android/app/src/main/java/dev/nako/android/ui/components/NakoSurfaces.kt`, `apps/android/app/src/main/java/dev/nako/android/ui/shell/NakoAppShell.kt`, `apps/android/app/src/main/java/dev/nako/android/ui/browse/BrowseComponents.kt`, `apps/android/app/src/main/java/dev/nako/android/ui/browse/NakoBrowseShell.kt`, `apps/android/app/src/test/java/dev/nako/android/ui/theme/NakoArtworkAccentsTest.kt`
  Handoff: Preserve current Public Client API clients and Media3 player
  boundaries; do not implement screen redesigns beyond what is needed to route
  through the new shell.

## M2 — Home And Browse V2

- [x] AME-030 [owner=codex] [deps=AME-020] [scope=apps/android/app/src/main/java/dev/nako/android/ui/browse/HomeScreen.kt,apps/android/app/src/main/java/dev/nako/android/ui/browse/LibrariesScreen.kt,apps/android/app/src/main/java/dev/nako/android/ui/browse/BrowseFacetRouteContent.kt]
  Goal: Implement V2 Home, Libraries, Library Detail, and Browse Facet Result
  with artwork-led rails, structural library anchors, API-backed facets only,
  and no fake Continue Watching.
  Validation: `apps\android\gradlew.bat -p apps\android :app:testDebugUnitTest --no-daemon`; `apps\android\gradlew.bat -p apps\android :app:assembleDebug --no-daemon`; `git diff --check`
  Review: Use review-workstream before accepting completion.
  Evidence: `apps/android/app/src/main/java/dev/nako/android/ui/browse/HomeScreen.kt`, `apps/android/app/src/main/java/dev/nako/android/ui/browse/LibrariesScreen.kt`, `apps/android/app/src/main/java/dev/nako/android/ui/browse/BrowseFacetRouteContent.kt`
  Handoff: Split Public Client API gaps rather than inventing local filters.

## M3 — Detail And Source Picker V2

- [x] AME-040 [owner=codex] [deps=AME-020] [scope=apps/android/app/src/main/java/dev/nako/android/ui/screens/detail,apps/android/app/src/main/java/dev/nako/android/ui/screens/sourcepicker]
  Goal: Implement the V2 Media Item Detail and Source / Version Picker as a
  playback decision surface with clear Play/Resume, source summary, metadata
  relationships, and playback-mode consequences.
  Validation: `apps\android\gradlew.bat -p apps\android :app:testDebugUnitTest --no-daemon`; `apps\android\gradlew.bat -p apps\android :app:assembleDebug --no-daemon`; `git diff --check`
  Review: Use review-workstream before accepting completion.
  Evidence: `apps/android/app/src/main/java/dev/nako/android/ui/screens/detail/MediaItemDetailRoute.kt`, `apps/android/app/src/main/java/dev/nako/android/ui/screens/sourcepicker/SourcePickerScreen.kt`, `apps/android/app/src/main/java/dev/nako/android/ui/browse/NakoBrowseShell.kt`, `apps/android/app/src/test/java/dev/nako/android/ui/screens/sourcepicker/SourcePickerDisplayModelTest.kt`
  Handoff: The detail screen now shows an artwork-led playback decision hero,
  device-local resume as explicitly local, metadata relationship entry points,
  and a source/version picker surface that explains Direct, Remux, HLS, and
  Transcode consequences without exposing locators or parsing HLS playlists.

## M4 — Player And Settings V2

- [x] AME-050 [owner=codex] [deps=AME-020] [scope=apps/android/app/src/main/java/dev/nako/android/ui/screens/player,apps/android/app/src/main/java/dev/nako/android/ui/screens/settings]
  Goal: Implement V2 Player, playback error sheet, Settings Home, and Server
  Profile with restrained settings chrome, safe diagnostics, and reliable
  player exit behavior.
  Validation: `apps\android\gradlew.bat -p apps\android :app:testDebugUnitTest --no-daemon`; `apps\android\gradlew.bat -p apps\android :app:assembleDebug --no-daemon`; `git diff --check`; emulator install/launch screenshot sanity check.
  Review: Use review-workstream before accepting completion.
  Evidence: `apps/android/app/src/main/java/dev/nako/android/ui/screens/player/PlaybackPlayerRoute.kt`, `apps/android/app/src/main/java/dev/nako/android/ui/screens/player/PlayerPresentation.kt`, `apps/android/app/src/main/java/dev/nako/android/ui/screens/settings/SettingsScreens.kt`, `apps/android/app/src/main/java/dev/nako/android/ui/screens/settings/SettingsPresentation.kt`, `apps/android/app/src/test/java/dev/nako/android/ui/screens/player/PlayerPresentationTest.kt`, `apps/android/app/src/test/java/dev/nako/android/ui/screens/settings/SettingsPresentationTest.kt`
  Handoff: Player now has V2 overlay chrome, loading status, sanitized playback
  error sheet, and preserved Media3/session cleanup behavior. Settings and
  Server Profile now live under `ui/screens/settings` with restrained grouped
  surfaces and sanitized diagnostics. Advanced gestures, PiP, downloads, and
  external player remain out of scope.

## M5 — Verification And Closeout

- [x] AME-060 [owner=planner] [deps=AME-030,AME-040,AME-050] [scope=docs/workstreams/android-material-expressive-ui,apps/android]
  Goal: Verify the completed V2 UI rewrite, update evidence, close or split
  follow-on work.
  Validation: Android unit tests, debug assemble, `cargo fmt --all -- --check`,
  `cargo nextest run --workspace --no-fail-fast`, and `git diff --check`.
  Review: Use review-workstream and verify-rust-workstream before closeout.
  Evidence: `EVIDENCE_AND_GATES.md`, `HANDOFF.md`, `WORKSTREAM.json`.
  Handoff: Completed. Final gates passed on 2026-05-18. Remaining API gaps,
  player feature depth, clipboard API migration, and deferred V3 exploration
  are recorded as follow-ons.
