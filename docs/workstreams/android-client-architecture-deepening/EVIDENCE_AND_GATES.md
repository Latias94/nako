# Android Client Architecture Deepening — Evidence And Gates

Status: Draft
Last updated: 2026-05-22

## Smallest Current Repro

```powershell
apps\android\gradlew.bat -p apps\android :app:testDebugUnitTest --tests dev.taru.android.connection.TaruConnectionClientTest --no-daemon --no-parallel
```

This gate proves the Android test harness, generated SDK/UniFFI binding path,
and one Public Client runtime entry point still compile before the first runtime
seam change.

## Gate Set

### Targeted Iteration Gates

Use the narrowest relevant gate while implementing each task:

```powershell
apps\android\gradlew.bat -p apps\android :app:testDebugUnitTest --tests dev.taru.android.connection.* --no-daemon --no-parallel
apps\android\gradlew.bat -p apps\android :app:testDebugUnitTest --tests dev.taru.android.browse.* --no-daemon --no-parallel
apps\android\gradlew.bat -p apps\android :app:testDebugUnitTest --tests dev.taru.android.playback.* --no-daemon --no-parallel
apps\android\gradlew.bat -p apps\android :app:testDebugUnitTest --tests dev.taru.android.userplayback.* --no-daemon --no-parallel
apps\android\gradlew.bat -p apps\android :app:testDebugUnitTest --tests dev.taru.android.ui.browse.* --no-daemon --no-parallel
apps\android\gradlew.bat -p apps\android :app:testDebugUnitTest --tests dev.taru.android.ui.screens.player.* --no-daemon --no-parallel
```

### Android Package Gate

```powershell
apps\android\gradlew.bat -p apps\android :app:testDebugUnitTest --no-daemon --no-parallel
```

### Android Build Gate

```powershell
apps\android\gradlew.bat -p apps\android :app:assembleDebug -PtaruRustAndroidAbis=x86_64 --no-daemon --no-parallel
```

Use focused ABI selection for local iteration. Broaden ABI coverage only for
release/package validation or when native packaging changes.

### Local Validation / Smoke Gate

```powershell
apps\android\scripts\Validate-AndroidLocal.ps1
```

Run near closeout when the local Android/emulator/smoke environment is
available. Record skipped prerequisites explicitly instead of implying smoke
passed.

### Rust Gate For Core/UniFFI Touches

Only required if a task changes Rust client-core or UniFFI crates:

```powershell
cargo fmt --package taru-client-core --package taru-client-uniffi --check
cargo nextest run -p taru-client-core --no-fail-fast
cargo nextest run -p taru-client-uniffi --no-fail-fast
```

### Hygiene Gate

```powershell
git diff --check
python -m json.tool docs/workstreams/android-client-architecture-deepening/WORKSTREAM.json > $null
```

### Review Gate

Run a workstream/code-quality review before accepting task or lane completion.
Record blocking findings, missing gates, and residual risks in this file or in
`HANDOFF.md`.

## Evidence Anchors

- `docs/workstreams/android-client-architecture-deepening/DESIGN.md`
- `docs/workstreams/android-client-architecture-deepening/TODO.md`
- `docs/workstreams/android-client-architecture-deepening/MILESTONES.md`
- `docs/workstreams/android-client-architecture-deepening/HANDOFF.md`
- `apps/android/app/src/main/java/dev/taru/android/connection/PublicClientApiExecutor.kt`
- `apps/android/app/src/main/java/dev/taru/android/browse/TaruBrowseClient.kt`
- `apps/android/app/src/main/java/dev/taru/android/playback/TaruPlaybackClient.kt`
- `apps/android/app/src/main/java/dev/taru/android/userplayback/TaruUserPlaybackClient.kt`
- `apps/android/app/src/main/java/dev/taru/android/ui/browse/BrowseSession.kt`
- `apps/android/app/src/main/java/dev/taru/android/ui/browse/BrowseShellHost.kt`
- `apps/android/app/src/main/java/dev/taru/android/ui/screens/player/PlaybackPlayerRoute.kt`
- `apps/android/app/src/main/java/dev/taru/android/ui/screens/player/PlayerRouteHost.kt`
- `apps/android/app/src/main/java/dev/taru/android/ui/components/TaruSurfaces.kt`
- `apps/android/app/src/main/java/dev/taru/android/ui/browse/BrowseComponents.kt`
- `apps/android/app/src/main/java/dev/taru/android/ui/browse/ClientBrowseDataSource.kt`

## Evidence Log

### ACAD-010 — Lane Open

Status: DONE
Date: 2026-05-22
Evidence:

- `DESIGN.md` records the target state and non-goals.
- `TODO.md` splits the lane into vertical architecture slices.
- `MILESTONES.md`, `EVIDENCE_AND_GATES.md`, `WORKSTREAM.json`, and
  `HANDOFF.md` were created for durable continuation.

Notes:

- No production code was changed by ACAD-010.
- This lane intentionally follows closed Android refactor lanes instead of
  reopening them.

### ACAD-020 — Client Runtime And Request Execution Seam

Status: DONE
Date: 2026-05-22
Evidence:

- Added `apps/android/app/src/main/java/dev/taru/android/connection/PublicClientRuntime.kt`.
- Added `apps/android/app/src/test/java/dev/taru/android/connection/PublicClientRuntimeTest.kt`.
- Migrated these route-family clients to `PublicClientRuntime`:
  - `apps/android/app/src/main/java/dev/taru/android/connection/TaruConnectionClient.kt`
  - `apps/android/app/src/main/java/dev/taru/android/browse/TaruBrowseClient.kt`
  - `apps/android/app/src/main/java/dev/taru/android/playback/TaruPlaybackClient.kt`
  - `apps/android/app/src/main/java/dev/taru/android/userplayback/TaruUserPlaybackClient.kt`
- Deleted obsolete base URL/path/auth JSON helpers from
  `PublicClientApiExecutor`; it is now the lower-level transport/error/redaction
  executor behind the runtime seam.
- Updated `ClientBrowseDataSourceTest` to assert the Rust-core/Public Client
  API search descriptor encoding currently used by Android (`%20` for spaces).

Behavior proven:

- Missing access keys are rejected before route-family request builders run.
- Authenticated JSON execution decodes, transforms, and returns redacted safe
  request previews.
- Connection core execution preserves Rust-core safe previews for successful
  steps and sanitized transport failures.
- Browse, Playback, User Playback State, and Connection clients still preserve
  route-family product diagnostics, token redaction, and Public Client API route
  behavior.

Validation:

```powershell
apps\android\gradlew.bat -p apps\android :app:testDebugUnitTest --tests dev.taru.android.connection.PublicClientRuntimeTest --tests dev.taru.android.connection.TaruConnectionClientTest --tests dev.taru.android.browse.TaruBrowseClientTest --tests dev.taru.android.playback.TaruPlaybackClientTest --tests dev.taru.android.userplayback.TaruUserPlaybackClientTest --no-daemon --no-parallel
```

Result: PASS on 2026-05-22.

```powershell
apps\android\gradlew.bat -p apps\android :app:testDebugUnitTest --tests dev.taru.android.ui.browse.ClientBrowseDataSourceTest.search* --no-daemon --no-parallel
```

Result: PASS on 2026-05-22 after updating the stale query-encoding
expectation.

```powershell
apps\android\gradlew.bat -p apps\android :app:testDebugUnitTest --no-daemon --no-parallel
```

Result: PASS on 2026-05-22.

```powershell
git diff --check
python -m json.tool docs/workstreams/android-client-architecture-deepening/WORKSTREAM.json > $null
```

Result: PASS on 2026-05-22. Git emitted CRLF normalization warnings for edited
Kotlin files, but no whitespace errors.

### ACAD-030 — Browse Effects And Route Loading Ownership

Status: DONE
Date: 2026-05-22
Evidence:

- Added `apps/android/app/src/main/java/dev/taru/android/ui/browse/BrowseSessionEffects.kt`.
- Added `apps/android/app/src/test/java/dev/taru/android/ui/browse/BrowseSessionEffectCoordinatorTest.kt`.
- Updated `BrowseShellHost` to delegate saveable-state publication and
  route-displayed load-intent execution to `BrowseSessionEffectCoordinator`.
- Updated `BrowseRouteLoadingSession` so `TaruRoute.Player` is a transient
  non-load intent instead of being treated like a non-loadable route that clears
  previous detail/source/playback state.
- Strengthened `BrowseShellHostTest` and `BrowseSessionLoadingTest` around
  player-route transient behavior and prior detail/source preservation.

Behavior proven:

- Initial host startup publishes saveable state and emits a single initial
  route-displayed load intent.
- Route changes become explicit coordinator load intents without duplicate
  loads for repeated state emissions.
- User actions publish the freshest state and still return direct async jobs
  when the underlying `BrowseSession` action has one.
- Player routes are saved/restored transiently while retaining the previous
  detail/source state at runtime.
- Existing stale-response protection remains in `BrowseRouteStatePolicy` and
  the route loading sessions; focused stale route tests still pass.

Validation:

```powershell
apps\android\gradlew.bat -p apps\android :app:testDebugUnitTest --tests dev.taru.android.ui.browse.BrowseSessionEffectCoordinatorTest --tests dev.taru.android.ui.browse.BrowseShellHostTest --tests dev.taru.android.ui.browse.BrowseSessionLoadingTest --no-daemon --no-parallel
```

Result: PASS on 2026-05-22.

```powershell
apps\android\gradlew.bat -p apps\android :app:testDebugUnitTest --tests dev.taru.android.ui.browse.* --no-daemon --no-parallel
```

Result: PASS on 2026-05-22.

```powershell
apps\android\gradlew.bat -p apps\android :app:testDebugUnitTest --no-daemon --no-parallel
```

Result: PASS on 2026-05-22.

```powershell
git diff --check
python -m json.tool docs/workstreams/android-client-architecture-deepening/WORKSTREAM.json > $null
```

Result: PASS on 2026-05-22. Git emitted CRLF normalization warnings for edited
Kotlin files, but no whitespace errors.

### ACAD-040 — Android Player Runtime Seam

Status: DONE
Date: 2026-05-22
Evidence:

- Added `apps/android/app/src/main/java/dev/taru/android/ui/screens/player/PlaybackSessionRuntime.kt`.
- Added `apps/android/app/src/main/java/dev/taru/android/ui/screens/player/PlaybackSessionRuntimeFactory.kt`.
- Added `apps/android/app/src/test/java/dev/taru/android/ui/screens/player/PlaybackSessionRuntimeTest.kt`.
- Updated `PlayerRouteHost` to implement `PlaybackSessionRuntime`.
- Moved `PlayerRouteEngine` and listener contracts behind the runtime seam.
- Updated `PlaybackPlayerRoute` to depend on `PlaybackSessionRuntimeFactory`
  instead of directly constructing Media3 engines, token-backed exit runners,
  or playback exit coordinators.
- Updated app and preview wiring to create an Android runtime factory before
  creating the route renderer.

Behavior proven:

- `PlaybackSessionRuntime` owns prepare/retry/back/dispose orchestration,
  player event mapping, and idempotent exit-effect dispatch.
- `AndroidPlaybackSessionRuntimeFactory` reads access tokens only while
  creating a playback runtime and does not expose bearer tokens in runtime
  debug output.
- Existing player session, player route host, player presentation, playback
  client, and playback exit-effect tests still pass.
- `PlaybackPlayerRoute` is now primarily rendering, `PlayerView` binding,
  back/retry UI dispatch, and clipboard UI; runtime construction has moved out
  of the Composable route body.
- MediaSession, PiP, Cast, and track-selection are intentionally left as
  follow-ons behind the runtime seam.

Validation:

```powershell
apps\android\gradlew.bat -p apps\android :app:testDebugUnitTest --tests dev.taru.android.ui.screens.player.PlaybackSessionRuntimeTest --tests dev.taru.android.ui.screens.player.PlayerRouteHostTest --tests dev.taru.android.ui.screens.player.PlayerSessionTest --no-daemon --no-parallel
```

Result: PASS on 2026-05-22.

```powershell
apps\android\gradlew.bat -p apps\android :app:testDebugUnitTest --tests dev.taru.android.ui.screens.player.* --tests dev.taru.android.playback.* --tests dev.taru.android.player.* --no-daemon --no-parallel
```

Result: PASS on 2026-05-22.

```powershell
apps\android\gradlew.bat -p apps\android :app:testDebugUnitTest --no-daemon --no-parallel
```

Result: PASS on 2026-05-22.

```powershell
git diff --check
python -m json.tool docs/workstreams/android-client-architecture-deepening/WORKSTREAM.json > $null
```

Result: PASS on 2026-05-22. Git emitted CRLF normalization warnings for edited
Kotlin files, but no whitespace errors.

### ACAD-050 — UI Design-System And Screen Modularization

Status: DONE
Date: 2026-05-22
Evidence:

- Updated cross-screen UI callers to import generic design-system primitives
  directly from `dev.taru.android.ui.components`:
  - `TaruScreenColumn`
  - `TaruSectionHeader`
  - `TaruSurfaceCard`
  - `TaruStatusChip`
  - `TaruStatusPill`
  - `TaruIconBadge`
  - `TaruArtworkBackdrop`
  - `TaruPressableScale`
- Removed obsolete pass-through wrappers from
  `apps/android/app/src/main/java/dev/taru/android/ui/browse/BrowseComponents.kt`
  so `BrowseComponents` now focuses on Taru media/browse presentation such as
  library cards, media rows, relationship cards, state cards, and browse copy.
- Added
  `apps/android/app/src/main/java/dev/taru/android/ui/screens/sourcepicker/SourcePickerPresentation.kt`
  for Source Picker display models, playback-mode copy, resume copy, source
  selection, and probe fact labels.
- Added
  `apps/android/app/src/main/java/dev/taru/android/ui/screens/detail/DetailPresentation.kt`
  for Detail metadata targets, credit relationship rows, hero facts, hierarchy
  copy, and collection copy.
- Reduced route-composable files:
  - `SourcePickerScreen.kt` no longer owns its testable display-model/copy
    builders.
  - `MediaItemDetailRoute.kt` no longer owns its testable presentation/copy
    builders.
  - player/detail/settings/person/relationship routes no longer depend on
    browse-package wrappers for generic design-system primitives.

Behavior proven:

- Existing Source Picker display model tests still prove token/path-safe source
  facts, selection accessibility copy, playback-mode copy, probe fact labels,
  and resume copy.
- Existing Detail tests still prove stable person-detail targets and
  viewer-facing gap copy without leaking API/internal terms.
- Existing settings/player/relationship presentation tests still pass after
  switching to direct design-system component imports.
- Full browse UI/session and screen tests still pass.
- Full Android JVM tests still pass.

Validation:

```powershell
apps\android\gradlew.bat -p apps\android :app:testDebugUnitTest --tests dev.taru.android.ui.screens.sourcepicker.SourcePickerDisplayModelTest --tests dev.taru.android.ui.screens.detail.MediaItemDetailRouteTest --tests dev.taru.android.ui.screens.settings.SettingsPresentationTest --tests dev.taru.android.ui.screens.player.PlayerPresentationTest --tests dev.taru.android.ui.screens.relationship.RelationshipIndexRouteTest --no-daemon --no-parallel
```

Result: PASS on 2026-05-22.

```powershell
apps\android\gradlew.bat -p apps\android :app:testDebugUnitTest --tests dev.taru.android.ui.browse.* --tests dev.taru.android.ui.screens.detail.* --tests dev.taru.android.ui.screens.sourcepicker.* --tests dev.taru.android.ui.screens.person.* --tests dev.taru.android.ui.screens.relationship.* --tests dev.taru.android.ui.screens.settings.* --tests dev.taru.android.ui.screens.player.* --no-daemon --no-parallel
```

Result: PASS on 2026-05-22.

```powershell
apps\android\gradlew.bat -p apps\android :app:testDebugUnitTest --no-daemon --no-parallel
```

Result: PASS on 2026-05-22.

```powershell
git diff --check
python -m json.tool docs/workstreams/android-client-architecture-deepening/WORKSTREAM.json > $null
```

Result: PASS on 2026-05-22. Git emitted CRLF normalization warnings for edited
Kotlin files, but no whitespace errors.

### ACAD-060 — Home Section Read Model And Progressive Artwork

Status: DONE
Date: 2026-05-22
Evidence:

- Updated `apps/android/app/src/main/java/dev/taru/android/ui/browse/BrowseModels.kt`
  so `BrowseUiState.Content` now carries a `HomeReadModel`.
- Added section states for Media Libraries, Visible Titles, Continue Watching,
  and Managed Artwork:
  - `HomeSectionState.NotRequested`
  - `HomeSectionState.Available`
  - `HomeSectionState.Unavailable`
  - `HomeArtworkState`
  - `HomeArtworkFailure`
- Updated `apps/android/app/src/main/java/dev/taru/android/ui/browse/ClientBrowseDataSource.kt`
  to keep independently loaded Home sections instead of flattening every failure
  into one all-or-nothing state.
- Updated `HomeScreen.kt` and `LibrariesScreen.kt` to render section-level
  unavailable states and retry affordances instead of silently converting failed
  sections to empty lists.
- Added `ClientBrowseDataSourceTest` coverage for partial Home success,
  Continue Watching degraded state, and Managed Artwork failure.

Behavior proven:

- Home remains content when Media Libraries fail but Visible Titles succeed.
- The whole Home surface still fails when both primary browse sections fail.
- Continue Watching failure becomes a degraded Home section without leaking
  bearer tokens.
- Managed Artwork failures are recorded on the read model and do not block
  visible titles.
- Existing first-paint behavior remains covered by `BrowseSessionLoadingTest`:
  Home loading enters `BrowseUiState.Loading` before content publication.
- Home and Libraries screens no longer present failed sections as empty server
  results.

Validation:

```powershell
apps\android\gradlew.bat -p apps\android :app:testDebugUnitTest --tests dev.taru.android.ui.browse.ClientBrowseDataSourceTest --no-daemon --no-parallel --console=plain --rerun-tasks
```

Result: PASS on 2026-05-22.

```powershell
apps\android\gradlew.bat -p apps\android :app:testDebugUnitTest --tests dev.taru.android.ui.browse.* --no-daemon --no-parallel --console=plain
```

Result: PASS on 2026-05-22.

```powershell
apps\android\gradlew.bat -p apps\android :app:testDebugUnitTest --no-daemon --no-parallel --console=plain --rerun-tasks
```

Result: PASS on 2026-05-22.

```powershell
git diff --check
python -m json.tool docs/workstreams/android-client-architecture-deepening/WORKSTREAM.json > $null
```

Result: PASS on 2026-05-22. Git emitted CRLF normalization warnings for edited
Kotlin files, but no whitespace errors.

### ACAD-070 — Persistence, Lifecycle, And Build Hygiene Decision Sweep

Status: DONE
Date: 2026-05-22
Evidence:

- Added `androidx.lifecycle:lifecycle-runtime-compose` to
  `apps/android/gradle/libs.versions.toml` and
  `apps/android/app/build.gradle.kts`.
- Updated Compose `StateFlow` collection to use lifecycle-aware collection:
  - `apps/android/app/src/main/java/dev/taru/android/ui/TaruAndroidApp.kt`
  - `apps/android/app/src/main/java/dev/taru/android/ui/browse/TaruBrowseShell.kt`
  - `apps/android/app/src/main/java/dev/taru/android/ui/connection/TaruConnectionShell.kt`
  - `apps/android/app/src/main/java/dev/taru/android/ui/screens/player/PlaybackPlayerRoute.kt`
- Ran local validation through `Validate-AndroidLocal.ps1 -SkipSmoke`; Android
  JVM tests and debug assemble passed, smoke regression was explicitly skipped.

Behavior proven:

- App, Browse shell, Connection shell, and Player route no longer collect hot
  UI state from Composables without lifecycle awareness.
- JVM tests and debug APK assembly still pass after adding the lifecycle
  runtime dependency.
- UniFFI host binding generation and Android x86_64 debug packaging still run
  through the validation path.

Decision sweep:

- Keep current SharedPreferences-backed profile, playback preference, and
  device playback-position stores for this lane. They are small synchronous
  stores behind interfaces, covered by tests, and do not justify a DataStore or
  Room migration before larger product scopes such as offline/downloads.
- Keep `AndroidSecureTokenVault` on `EncryptedSharedPreferences` for now. It is
  isolated behind `TokenVault`, and replacing the deprecated AndroidX Security
  API should be a focused token-vault migration rather than mixed into UI
  lifecycle cleanup.
- Defer DataStore/Room migration to a named follow-on once settings/profile
  editing, offline playback, downloads, or library caching require richer
  persistence semantics.
- Defer Gradle module split. The single app module still has clear internal
  seams after ACAD-020 through ACAD-060, and module splitting would add build
  topology cost without current dependency pressure evidence.
- Defer CI/device-farm and multi-ABI policy hardening. The local validation
  script already records JVM, assemble, and smoke status; broader ABI/device
  coverage belongs in CI/release workflow planning.

Validation:

```powershell
apps\android\gradlew.bat -p apps\android :app:testDebugUnitTest --tests dev.taru.android.ui.browse.BrowseShellHostTest --tests dev.taru.android.ui.connection.ConnectionSessionTest --tests dev.taru.android.ui.screens.player.PlaybackSessionRuntimeTest --no-daemon --no-parallel --console=plain --rerun-tasks
```

Result: PASS on 2026-05-22.

```powershell
apps\android\scripts\Validate-AndroidLocal.ps1 -SkipSmoke
```

Result: PASS on 2026-05-22. Report paths:

- `apps/android/build/validation/20260522-145630/report.md`
- `apps/android/build/validation/20260522-145630/report.json`
- `apps/android/build/validation/20260522-145630/report.junit.xml`

The report shows:

- Android JVM tests: PASS.
- Android debug assemble: PASS.
- Android smoke regression: SKIPPED by explicit `-SkipSmoke`.

### ACAD-090 — Closeout

Status: DONE
Date: 2026-05-22
Evidence:

- Final review covered ACAD-020 through ACAD-070 against the workstream target
  state, repo guardrails, ADR 0026/0032 platform ownership boundaries, and
  token-safety requirements.
- `docs/workstreams/android-client-architecture-deepening/CLOSEOUT.md` records
  outcome, gates, review findings, residual risks, and follow-ons.
- `WORKSTREAM.json` is marked completed with no current task.

Review findings:

- Workstream Compliance: no blocking findings. All accepted tasks in
  `TODO.md` are complete, and larger persistence/build/product scopes are split
  as follow-ons instead of hidden in this lane.
- Code Quality: no blocking findings. The new seams are deep enough to keep
  generic execution, browse effects, player runtime, Home section state, and
  lifecycle collection policy out of broad Composable or route-family callers.
- Missing Gates: none blocking. Device/emulator smoke was intentionally skipped
  in `Validate-AndroidLocal.ps1 -SkipSmoke` and remains a follow-on/available
  local gate rather than an implied pass.
- Residual Risk: see `CLOSEOUT.md`.

Validation:

```powershell
apps\android\gradlew.bat -p apps\android :app:testDebugUnitTest --no-daemon --no-parallel --console=plain
```

Result: PASS on 2026-05-22.

```powershell
apps\android\gradlew.bat -p apps\android :app:assembleDebug -PtaruRustAndroidAbis=x86_64 --no-daemon --no-parallel --console=plain
```

Result: PASS on 2026-05-22.

```powershell
git diff --check
python -m json.tool docs/workstreams/android-client-architecture-deepening/WORKSTREAM.json > $null
```

Result: PASS on 2026-05-22. Git emitted CRLF normalization warnings for edited
Kotlin files, but no whitespace errors.

## Notes

Fresh verification is required before marking any implementation task, Codex
goal, or lane complete. Do not list commands without explaining the behavior
they cover.
