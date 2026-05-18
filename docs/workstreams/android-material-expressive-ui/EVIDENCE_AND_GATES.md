# Android Material Expressive UI — Evidence And Gates

Status: Active
Last updated: 2026-05-18

## Smallest Current Repro

```powershell
apps\android\gradlew.bat -p apps\android :app:testDebugUnitTest --no-daemon
apps\android\gradlew.bat -p apps\android :app:assembleDebug --no-daemon
git diff --check
```

## Gate Set

### Targeted Android Gate

```powershell
apps\android\gradlew.bat -p apps\android :app:testDebugUnitTest --no-daemon
```

This proves Android DTO/client unit tests and UI-adjacent behavior still pass.

### Android Build Gate

```powershell
apps\android\gradlew.bat -p apps\android :app:assembleDebug --no-daemon
```

This proves Compose, resources, manifest, Media3 dependencies, and debug
packaging still compile.

### Rust Workspace Gate

```powershell
cargo fmt --all -- --check
cargo nextest run --workspace --no-fail-fast
```

Run when server/public API/shared protocol files are touched, and before final
closeout.

### Diff Hygiene Gate

```powershell
git diff --check
```

This catches whitespace errors and unresolved patch artifacts.

### Review Gate

Use `review-workstream` before accepting each AME task and
`verify-rust-workstream` before closeout.

## Evidence Anchors

- `docs/workstreams/android-material-expressive-ui/DESIGN.md`
- `docs/workstreams/android-material-expressive-ui/TODO.md`
- `docs/workstreams/android-material-expressive-ui/MILESTONES.md`
- `docs/workstreams/android-material-expressive-ui/HANDOFF.md`
- `apps/android/app/src/main/java/dev/taru/android/ui/theme/`
- `apps/android/app/src/main/java/dev/taru/android/ui/components/`
- `apps/android/app/src/main/java/dev/taru/android/ui/shell/`
- `apps/android/app/src/main/java/dev/taru/android/ui/screens/`

## Evidence Log

- 2026-05-18: Workstream opened after merging `main` into
  `android-client-foundation`.
- 2026-05-18: `AME-020` completed with a new Material 3 theme/tokens layer,
  artwork-accent hook, shared UI surfaces, adaptive shell, browse-shell
  integration, and targeted JVM test coverage.
- 2026-05-18: `AME-030` completed V2 Home, Libraries, and Browse Facet Result
  surfaces. `HomeScreen` no longer shows fake Continue Watching or unsupported
  facet shortcuts; `LibrariesScreen` keeps library browsing structural and
  visible items media-led; `BrowseFacetRouteContent` presents API-backed
  relationship results and keeps unsupported families explicit API-gap states.
  Validation passed:
  `apps\android\gradlew.bat -p apps\android :app:testDebugUnitTest --no-daemon`;
  `apps\android\gradlew.bat -p apps\android :app:assembleDebug --no-daemon`;
  `git diff --check`.
- 2026-05-18: `AME-040` completed V2 Media Item Detail and Source / Version
  Picker surfaces. Detail now routes through `ui/screens/detail` with an
  artwork-led playback decision hero, explicit device-local resume wording,
  metadata relationship chips, Cast & Crew preview, and API-gap relationship
  rows. Source / Version selection now routes through `ui/screens/sourcepicker`
  and explains source choice plus Direct, Remux, HLS, and Transcode
  consequences without exposing Media Source locators or parsing HLS playlists.
  Added JVM coverage for source-picker display models and client-safe source
  facts. Validation passed:
  `apps\android\gradlew.bat -p apps\android :app:testDebugUnitTest --no-daemon`;
  `apps\android\gradlew.bat -p apps\android :app:assembleDebug --no-daemon`;
  `git diff --check`.
- 2026-05-18: `AME-050` completed V2 Player, Playback Error Sheet, Settings
  Home, and Server Profile surfaces. Player now routes through
  `ui/screens/player` with immersive overlay chrome, loading status, sanitized
  playback error recovery, Media3 PlayerView preservation, device-local
  position persistence, and existing playback-session cancellation behavior.
  Settings now routes through `ui/screens/settings` with restrained grouped
  surfaces, token-safe access copy, server profile switching, and sanitized
  diagnostics. Added JVM presentation tests for player error diagnostics,
  local resume wording, and settings diagnostics safety. Validation passed:
  `apps\android\gradlew.bat -p apps\android :app:testDebugUnitTest --no-daemon`;
  `apps\android\gradlew.bat -p apps\android :app:assembleDebug --no-daemon`;
  `git diff --check`; emulator install/launch screenshot sanity check for
  Home, Settings, and Server Profile surfaces.
