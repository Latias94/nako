# Android Client Architecture Deepening — Handoff

Status: Draft
Last updated: 2026-05-22

## Current State

The workstream is closed for the requested fearless Android client
architecture-deepening pass. ACAD-010, ACAD-020, ACAD-030, ACAD-040,
ACAD-050, ACAD-060, ACAD-070, and ACAD-090 are complete.

This lane is a follow-on to earlier closed Android lanes, especially
`android-fearless-client-refactor`, `generated-sdk-runtime-ownership`,
`android-rust-core-runtime-hardening`, `android-browse-shell-session-host`,
`android-player-session-architecture`, `android-player-route-host`, and
`android-material-expressive-ui`.

## Completed Tasks

### ACAD-010 — Scope And Evidence Freeze

Opened `docs/workstreams/android-client-architecture-deepening/` with design,
milestones, gates, task ledger, handoff, and workstream metadata.

### ACAD-020 — Client Runtime And Request Execution Seam

Introduced `PublicClientRuntime` as the Android-owned runtime seam above
`PublicClientApiExecutor`. Route-family clients now use it for authenticated
request validation, transport execution, JSON decode, safe request propagation,
and core-response execution while keeping product diagnostics local.

### ACAD-030 — Browse Effects And Route Loading Ownership

Introduced `BrowseSessionEffectCoordinator` and `BrowseSessionLoadIntent`.
`BrowseShellHost` now wires the coordinator to `BrowseSession` instead of
owning displayed-route tracking and saveable-state publication inline.

Ownership after ACAD-030:

- `BrowseSessionEffectCoordinator` owns saveable-state publication timing,
  route-display deduplication, and load-intent emission.
- `BrowseShellHost` owns Android/runtime wiring and host lifetime.
- `BrowseSession` remains the deterministic state machine and async action
  entry point.
- `BrowseRouteLoadingSession` owns route-family loading and treats
  `NakoRoute.Player` as a transient non-load intent.
- `BrowseRouteStatePolicy` continues to own stale-response invalidation and
  route-family state clearing.

### ACAD-040 — Android Player Runtime Seam

Introduced `PlaybackSessionRuntime` and `PlaybackSessionRuntimeFactory`.
`PlayerRouteHost` now implements the runtime interface, and
`AndroidPlaybackSessionRuntimeFactory` creates route-scoped runtimes with a
Media3 engine and token-safe exit-effect runner.

Ownership after ACAD-040:

- `PlaybackSessionRuntime` owns attach/prepare/retry/back/dispose, player
  state, `Player` access for `PlayerView`, and exit-effect dispatch.
- `PlaybackSessionRuntimeFactory` owns creation of route-scoped playback
  runtimes.
- `AndroidPlaybackSessionRuntimeFactory` owns Android dependency composition:
  app context, token vault reads, Media3 engine factory, playback clients,
  position store, and exit-effect scope.
- `PlaybackPlayerRoute` now owns route rendering, `PlayerView` binding,
  retry/back UI dispatch, and clipboard UI only.
- MediaSession, PiP, Cast, and track-selection remain follow-ons behind the
  runtime seam.

### ACAD-050 — UI Design-System And Screen Modularization

Removed the browse-package pass-through layer for generic design-system
primitives. Cross-screen callers now import `NakoScreenColumn`,
`NakoSectionHeader`, `NakoSurfaceCard`, `NakoStatusChip`, `NakoStatusPill`,
`NakoIconBadge`, `NakoArtworkBackdrop`, and `NakoPressableScale` directly from
`dev.nako.android.ui.components`.

`BrowseComponents` now focuses on Nako media/browse presentation: page titles,
state cards, library cards, media rows, relationship cards, and browse copy.
It no longer re-exports generic design-system components under browse-owned
names.

Extracted testable presentation/copy builders:

- `SourcePickerPresentation.kt` owns Source Picker display models,
  playback-mode copy, source selection, resume copy, and probe fact labels.
- `DetailPresentation.kt` owns Detail metadata targets, credit relationship
  rows, hero facts, hierarchy copy, and collection copy.

The existing Source Picker and Detail presentation tests cover the extracted
logic, including token/path-safe source facts, selection accessibility copy,
playback-mode copy, resume copy, stable person-detail targets, and viewer-facing
gap copy.

Remaining large files and reason to keep or defer:

- `HomeScreen.kt`: now has section-specific rendering helpers for the ACAD-060
  Home read model. Further extraction should be by product concept, for example
  Home hero versus Home sections, not by file size alone.
- `SettingsScreens.kt`: still combines route surfaces with connection/profile
  settings presentation. It no longer depends on browse wrappers. A future
  settings-specific split should happen only when settings persistence or
  profile editing changes.
- `PlaybackPlayerRoute.kt`: reduced by ACAD-040 runtime extraction and now
  imports design primitives directly. MediaSession/PiP/Cast/track-selection are
  follow-ons behind the runtime seam.
- `BrowseComponents.kt`: remains a media/browse presentation module. Further
  splits should be by product concept, for example media cards versus library
  cards, not by file size alone.

### ACAD-060 — Home Section Read Model And Progressive Artwork

Introduced `HomeReadModel` as the Home surface read model behind
`BrowseUiState.Content`.

Ownership after ACAD-060:

- `HomeReadModel` owns the aggregate Home sections exposed to UI.
- `HomeSectionState` represents independently available, unavailable, or
  intentionally not requested Home sections.
- `HomeArtworkState` represents Managed Artwork enrichment separately from the
  primary Media Item list.
- `ClientBrowseDataSource.loadHome()` owns section assembly and token-safe
  degraded diagnostics.
- `HomeScreen` and `LibrariesScreen` render section-level unavailable states
  instead of treating failed sections as empty server results.

Behavior after ACAD-060:

- Media Libraries and Visible Titles are primary sections. If either succeeds,
  Home can render partial content. If both fail, Home remains a full failure.
- Continue Watching is represented as its own section. A failure degrades only
  that section.
- Managed Artwork is progressive enrichment. Per-item artwork failure is
  recorded and surfaced without blocking visible titles.
- Existing convenience accessors on `BrowseUiState.Content` remain for older
  call sites and tests, but new code should prefer `state.home`.

### ACAD-070 — Persistence, Lifecycle, And Build Hygiene Decision Sweep

Implemented the small safe lifecycle cleanup and recorded defer/split decisions
for larger persistence/build topics.

Changes after ACAD-070:

- Added `androidx.lifecycle:lifecycle-runtime-compose`.
- Replaced Compose `collectAsState()` calls with
  `collectAsStateWithLifecycle()` in:
  - `NakoAndroidApp.kt`
  - `NakoBrowseShell.kt`
  - `NakoConnectionShell.kt`
  - `PlaybackPlayerRoute.kt`
- Verified Android JVM tests and debug assemble through
  `Validate-AndroidLocal.ps1 -SkipSmoke`.

Decisions after ACAD-070:

- Keep current SharedPreferences-backed profile, playback preference, and
  device playback-position stores for this lane. They remain behind interfaces
  and are adequate until product scope demands richer persistence.
- Keep `AndroidSecureTokenVault` isolated behind `TokenVault`; a future
  replacement for deprecated AndroidX Security APIs should be a focused
  token-vault migration.
- Split DataStore/Room migration, Gradle module split, CI/device-farm
  hardening, and multi-ABI release policy into follow-ons when the product scope
  justifies them.

## Active Task

None. Closeout is complete.

Review result:

- Workstream Compliance: no blocking findings.
- Code Quality: no blocking findings.
- Missing Gates: none blocking.
- Residual risks and follow-ons are recorded in `CLOSEOUT.md`.

Evidence: ACAD-020 through ACAD-090 evidence recorded in
`EVIDENCE_AND_GATES.md`.

## Decisions Since Last Update

- Create a new workstream `android-client-architecture-deepening` instead of
  reopening closed Android refactor lanes.
- Keep the lane broad enough to finish the architecture findings from the
  2026-05-22 review, but split into vertical tasks with explicit gates.
- Do not split Gradle modules by default. Reconsider only after evidence shows
  dependency/build pressure.
- Do not move Android UI, Media3, token vault, profile persistence, or platform
  networking into Rust.
- ACAD-020 introduced `PublicClientRuntime`; obsolete base/path/auth helpers
  were deleted from `PublicClientApiExecutor`.
- `ClientBrowseDataSourceTest` was updated to match the Rust-core search
  descriptor's `%20` query encoding.
- ACAD-030 chose a small explicit coordinator seam rather than introducing an
  Android lifecycle framework dependency.
- ACAD-040 chose a runtime/factory seam rather than implementing MediaSession,
  PiP, Cast, or track-selection now. Those remain product follow-ons.
- ACAD-050 chose direct design-system imports over browse-package re-export
  wrappers. The browse package should own product presentation, not generic UI
  primitives.
- ACAD-050 deferred Home decomposition to ACAD-060 because Home's main problem
  is read-model shape, not only file size.
- ACAD-060 chose `HomeReadModel` and `HomeSectionState` inside the browse UI
  model layer rather than creating a new repository/cache layer before product
  persistence decisions are made.
- ACAD-060 preserves old `BrowseUiState.Content` convenience accessors for
  compatibility but treats `state.home` as the preferred interface for new Home
  and Libraries rendering.
- ACAD-060 does not fabricate totals or local section filters. It only renders
  counts and rows returned by current Public Client API contracts.
- ACAD-070 chose lifecycle-aware Compose collection as a safe in-lane cleanup.
- ACAD-070 intentionally deferred DataStore/Room migration, Gradle module
  splitting, CI/device-farm hardening, multi-ABI policy expansion, and
  `EncryptedSharedPreferences` replacement to focused follow-ons.
- ACAD-090 closed the lane and routed remaining product scopes to follow-ons in
  `CLOSEOUT.md`.

## Latest Validation

Passed on 2026-05-22:

```powershell
apps\android\gradlew.bat -p apps\android :app:testDebugUnitTest --no-daemon --no-parallel --console=plain
```

Passed on 2026-05-22:

```powershell
apps\android\gradlew.bat -p apps\android :app:assembleDebug -PnakoRustAndroidAbis=x86_64 --no-daemon --no-parallel --console=plain
```

Passed on 2026-05-22:

```powershell
apps\android\scripts\Validate-AndroidLocal.ps1 -SkipSmoke
```

Result report:

- `apps/android/build/validation/20260522-145630/report.md`
- `apps/android/build/validation/20260522-145630/report.json`
- `apps/android/build/validation/20260522-145630/report.junit.xml`

The report records Android JVM tests PASS, Android debug assemble PASS, and
Android smoke regression SKIPPED because `-SkipSmoke` was explicitly used.

Passed on 2026-05-22:

```powershell
git diff --check
python -m json.tool docs/workstreams/android-client-architecture-deepening/WORKSTREAM.json > $null
```

`git diff --check` emitted only CRLF normalization warnings for edited Kotlin
files.

## Blockers

None.

## Next Recommended Action

Ask the user for commit confirmation. Suggested commit message:

```text
refactor(android): deepen client runtime and UI architecture
```

## Guardrails

- Keep responses to the user in Chinese; keep code and technical docs in
  English.
- Prefer `apply_patch` for code edits.
- Do not use `git restore`, `git checkout`, `git reset`, stash, or destructive
  cleanup to remove changes that may belong to the user.
- Do not touch generated `output/` or `tmp/` directories unless the task
  explicitly requires generated artifacts.
- Keep bearer tokens, local source locators, local paths, FFmpeg command lines,
  and server internals out of UI, diagnostics, saved route state, and logs.
- Android owns UI, Media3, platform lifecycle, token/profile storage, and
  transport execution; Rust owns only portable client-core semantics endorsed by
  ADR 0032.
