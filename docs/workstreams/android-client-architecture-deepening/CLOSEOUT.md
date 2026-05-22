# Android Client Architecture Deepening — Closeout

Status: Closed
Closed on: 2026-05-22

## Outcome

This lane completed the requested fearless Android client architecture-deepening
pass. The Android app now has clearer ownership for Public Client runtime
execution, browse side effects, player lifecycle, design-system boundaries, Home
read models, and lifecycle-aware UI state collection.

Completed slices:

1. `PublicClientRuntime` now centralizes authenticated request validation,
   Android transport execution, generated SDK JSON decode, safe request preview
   propagation, and Rust-core response execution while route-family clients keep
   product diagnostics and DTO mapping local.
2. `BrowseSessionEffectCoordinator` now owns saveable-state publication and
   route-displayed load intents, leaving `BrowseSession` as deterministic state
   machinery and `BrowseShellHost` as Android wiring.
3. `PlaybackSessionRuntime` and `PlaybackSessionRuntimeFactory` now form the
   Android-owned player runtime seam. `PlaybackPlayerRoute` renders and binds the
   runtime instead of constructing Media3, token-backed exit effects, and route
   host internals inline.
4. Generic design-system primitives are imported directly from
   `dev.nako.android.ui.components`; obsolete browse-package pass-through
   wrappers were removed. Source Picker and Detail display/copy builders were
   extracted into test-covered presentation modules.
5. Home now uses `HomeReadModel`, `HomeSectionState`, and `HomeArtworkState` to
   represent Media Libraries, Visible Titles, Continue Watching, and Managed
   Artwork independently. Section-level degraded states no longer masquerade as
   empty server results.
6. App, Browse shell, Connection shell, and Player route now collect Compose
   `StateFlow` state with `collectAsStateWithLifecycle()`.
7. Persistence, Gradle module split, token-vault replacement, and CI/device
   policy questions were deliberately routed as follow-ons where they exceed the
   lane.

## Final Verification

Fresh closeout gates run on 2026-05-22:

```powershell
apps\android\gradlew.bat -p apps\android :app:testDebugUnitTest --no-daemon --no-parallel --console=plain
apps\android\gradlew.bat -p apps\android :app:assembleDebug -PnakoRustAndroidAbis=x86_64 --no-daemon --no-parallel --console=plain
git diff --check
python -m json.tool docs/workstreams/android-client-architecture-deepening/WORKSTREAM.json > $null
```

All gates passed. `git diff --check` emitted only CRLF normalization warnings
for edited Kotlin files and no whitespace errors.

Additional local validation run on 2026-05-22:

```powershell
apps\android\scripts\Validate-AndroidLocal.ps1 -SkipSmoke
```

Result: PASS. Report files:

- `apps/android/build/validation/20260522-145630/report.md`
- `apps/android/build/validation/20260522-145630/report.json`
- `apps/android/build/validation/20260522-145630/report.junit.xml`

The report records Android JVM tests PASS, Android debug assemble PASS, and
Android smoke regression SKIPPED by explicit `-SkipSmoke`.

## Review Findings

### Workstream Compliance

No blocking findings.

- ACAD-020 through ACAD-070 are marked complete in `TODO.md` with evidence.
- `EVIDENCE_AND_GATES.md`, `MILESTONES.md`, `HANDOFF.md`, and `WORKSTREAM.json`
  agree on completed task order and validation.
- The lane respected ADR 0026/0032: Android still owns transport, token/profile
  persistence, Compose UI, Media3, platform lifecycle, and player behavior.
- No server route shapes or Public Client API contracts were changed.

### Code Quality

No blocking findings.

- The new seams are deep modules rather than pass-through wrappers:
  `PublicClientRuntime`, `BrowseSessionEffectCoordinator`,
  `PlaybackSessionRuntime`, and `HomeReadModel` each hide behavior that would
  otherwise leak into multiple callers.
- Token-safety behavior remains explicit through safe request previews,
  redacted diagnostics, runtime factory ownership, and tests.
- UI wrappers were deleted only where direct design-system imports improved
  locality.
- Home section rendering now reflects degraded state rather than fabricating
  empty data.

### Missing Gates

None blocking.

- Device/emulator smoke was intentionally skipped in the local validation run
  using `-SkipSmoke`. This is recorded as skipped, not implied as passed.

## Residual Risks

- `AndroidSecureTokenVault` still uses deprecated AndroidX Security
  `EncryptedSharedPreferences`. It is isolated behind `TokenVault`, but should
  be replaced in a focused token-vault migration when prioritized.
- SharedPreferences remains the production store for profiles, playback
  preferences, and device playback positions. This is acceptable for the current
  client foundation, but DataStore/Room should be reconsidered when downloads,
  offline playback, local catalog cache, or richer migrations become product
  scope.
- Device smoke and CI device-farm coverage were not expanded by this lane. Local
  validation reports skipped smoke explicitly when no emulator gate is run.
- MediaSession, PiP, Cast, track selection, Android TV, downloads/offline, and
  external player handoff remain product follow-ons behind the new seams.
- Home section read models still depend on current Public Client API routes. If
  future sections need different semantics, split server/API work instead of
  inventing local totals or filters.

## Recommended Follow-ons

1. Token vault migration: replace deprecated AndroidX Security token storage
   behind `TokenVault` with a maintained Android credential/encrypted storage
   strategy and migration tests.
2. Persistence architecture: introduce DataStore or Room only when product scope
   demands richer profile/settings/cache/offline semantics.
3. Player capability expansion: implement MediaSession, PiP, Cast, audio/subtitle
   track selection, or external player handoff behind `PlaybackSessionRuntime`.
4. Device/CI policy: add emulator/device smoke, native UniFFI smoke, and
   multi-ABI packaging gates to CI when release workflow needs them.
5. Home/product sections: add new server/API read models for recommendations,
   collections, or richer Managed Artwork rather than local synthetic sections.
6. Optional UI modularization: split `HomeScreen.kt`, `SettingsScreens.kt`, or
   `BrowseComponents.kt` further only by product concept and with tests, not by
   file size alone.

## Commit Guidance

Suggested conventional commit after user confirmation:

```text
refactor(android): deepen client runtime and UI architecture
```
