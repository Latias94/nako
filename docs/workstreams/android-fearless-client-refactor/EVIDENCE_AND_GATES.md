# Android Fearless Client Refactor — Evidence And Gates

Status: Complete
Last updated: 2026-05-21

This file records validation commands and evidence anchors for the lane.

## Gate Policy

- Prefer focused Android JVM tests for each architecture slice.
- Run full Android JVM tests before claiming a slice is complete.
- Run debug assemble after changes that touch production Android code.
- Run smoke validation when UI navigation, playback, network behavior, or
  smoke-covered copy changes materially.
- Keep all evidence token-safe and locator-safe.
- Do not commit generated screenshots, smoke reports, APKs, fixture data, or
  build outputs.

## Baseline Commands

From repository root:

```powershell
git status --short --branch
git diff --check
```

From `apps/android`:

```powershell
.\gradlew.bat -p apps/android :app:testDebugUnitTest --no-daemon
.\gradlew.bat -p apps/android :app:assembleDebug --no-daemon
.\scripts\Validate-AndroidLocal.ps1 -SkipSmoke
.\scripts\Validate-AndroidLocal.ps1
```

## Slice Gates

### AFCR-010 Public Client API Adapter

Focused gates:

```powershell
apps/android/gradlew.bat -p apps/android :app:testDebugUnitTest --tests dev.taru.android.connection.* --no-daemon
apps/android/gradlew.bat -p apps/android :app:testDebugUnitTest --tests dev.taru.android.browse.* --no-daemon
apps/android/gradlew.bat -p apps/android :app:testDebugUnitTest --tests dev.taru.android.playback.* --no-daemon
apps/android/gradlew.bat -p apps/android :app:testDebugUnitTest --tests dev.taru.android.userplayback.* --no-daemon
```

Evidence:

- `apps/android/gradlew.bat -p apps/android :app:compileDebugUnitTestKotlin --no-daemon`
  — PASS on 2026-05-20 after introducing `PublicClientApiExecutor` and
  migrating route clients.
- `apps/android/gradlew.bat -p apps/android :app:testDebugUnitTest --tests dev.taru.android.connection.* --no-daemon`
  — PASS on 2026-05-20.
- `apps/android/gradlew.bat -p apps/android :app:testDebugUnitTest --tests dev.taru.android.browse.* --no-daemon`
  — PASS on 2026-05-20.
- `apps/android/gradlew.bat -p apps/android :app:testDebugUnitTest --tests dev.taru.android.playback.* --no-daemon`
  — PASS on 2026-05-20, including session-preflight HTTP error and API
  version regression coverage through the shared executor.
- `apps/android/gradlew.bat -p apps/android :app:testDebugUnitTest --tests dev.taru.android.userplayback.* --no-daemon`
  — PASS on 2026-05-20.
- `apps/android/gradlew.bat -p apps/android :app:testDebugUnitTest --no-daemon`
  — PASS on 2026-05-20.
- `git diff --check` — PASS on 2026-05-20.

### AFCR-020 Token-Safe Playback Launch

Focused gates:

```powershell
apps/android/gradlew.bat -p apps/android :app:testDebugUnitTest --tests dev.taru.android.player.* --no-daemon
apps/android/gradlew.bat -p apps/android :app:testDebugUnitTest --tests dev.taru.android.ui.screens.player.* --no-daemon
```

Evidence:

- `apps/android/gradlew.bat -p apps/android :app:compileDebugUnitTestKotlin --no-daemon`
  — PASS on 2026-05-20 after replacing route-level raw playback requests with
  token-safe descriptors.
- `apps/android/gradlew.bat -p apps/android :app:testDebugUnitTest --tests dev.taru.android.playback.* --tests dev.taru.android.player.* --tests dev.taru.android.ui.screens.player.* --no-daemon`
  — PASS on 2026-05-20. Covers token-safe playback descriptors, launch
  creation, player diagnostics, playback start, and exit effects.
- `apps/android/gradlew.bat -p apps/android :app:testDebugUnitTest --tests dev.taru.android.ui.browse.* --no-daemon`
  — PASS on 2026-05-20. Covers transient Player route save/restore behavior
  and browse start-playback integration.
- `apps/android/gradlew.bat -p apps/android :app:testDebugUnitTest --no-daemon`
  — PASS on 2026-05-20.
- `git diff --check` — PASS on 2026-05-20; only Git line-ending warnings were
  reported.

### AFCR-030 Browse State Deepening

Focused gates:

```powershell
apps/android/gradlew.bat -p apps/android :app:testDebugUnitTest --tests dev.taru.android.ui.browse.* --no-daemon
```

Evidence:

- `apps/android/gradlew.bat -p apps/android :app:compileDebugUnitTestKotlin --no-daemon`
  — PASS on 2026-05-21 after extracting browse state modules.
- `apps/android/gradlew.bat -p apps/android :app:testDebugUnitTest --tests dev.taru.android.ui.browse.* --no-daemon`
  — PASS on 2026-05-21. Covers existing browse session behavior plus
  module-level route state policy regressions.
- `apps/android/gradlew.bat -p apps/android :app:testDebugUnitTest --no-daemon`
  — PASS on 2026-05-21.
- `git diff --check` — PASS on 2026-05-21; only Git line-ending warnings were
  reported.

### AFCR-040 Transport And Network Security

Focused gates:

```powershell
apps/android/gradlew.bat -p apps/android :app:testDebugUnitTest --tests dev.taru.android.connection.* --no-daemon
```

Evidence:

- `apps/android/gradlew.bat -p apps/android :app:compileDebugUnitTestKotlin --no-daemon`
  — PASS on 2026-05-21 after transport and cleartext policy hardening.
- `apps/android/gradlew.bat -p apps/android :app:testDebugUnitTest --tests dev.taru.android.connection.* --no-daemon`
  — PASS on 2026-05-21. Covers production cleartext rejection, explicit local
  development cleartext allowance, final transport guard behavior, and existing
  connection diagnostics.
- `apps/android/gradlew.bat -p apps/android :app:testDebugUnitTest --no-daemon`
  — PASS on 2026-05-21.
- `git diff --check` — PASS on 2026-05-21; only Git line-ending warnings were
  reported.

### AFCR-050 Paging

Focused gates:

```powershell
apps/android/gradlew.bat -p apps/android :app:testDebugUnitTest --tests dev.taru.android.ui.browse.* --no-daemon
apps/android/gradlew.bat -p apps/android :app:testDebugUnitTest --tests dev.taru.android.browse.* --no-daemon
```

Evidence:

- `apps/android/gradlew.bat -p apps/android :app:compileDebugUnitTestKotlin --no-daemon`
  — PASS on 2026-05-21 after adding shared paging state, load-more actions,
  and debug cleartext `BuildConfig` wiring.
- `apps/android/gradlew.bat -p apps/android :app:testDebugUnitTest --tests dev.taru.android.ui.browse.* --no-daemon`
  — PASS on 2026-05-21. Covers Search, relationship index, and facet load-more
  behavior; next pages are derived only from Public Client API `limit`,
  `offset`, and `returned`.
- `apps/android/gradlew.bat -p apps/android :app:testDebugUnitTest --tests dev.taru.android.browse.* --tests dev.taru.android.connection.* --no-daemon`
  — PASS on 2026-05-21. Covers browse query construction plus the AFCR-040
  debug/local-development cleartext policy guard.
- `apps/android/gradlew.bat -p apps/android :app:testDebugUnitTest --no-daemon`
  — PASS on 2026-05-21.
- `git diff --check` — PASS on 2026-05-21; only Git line-ending warnings were
  reported.

### AFCR-060 UI Copy, Accessibility, Localization

Focused gates:

```powershell
apps/android/gradlew.bat -p apps/android :app:compileDebugUnitTestKotlin --no-daemon
apps/android/gradlew.bat -p apps/android :app:testDebugUnitTest --tests dev.taru.android.ui.screens.settings.* --tests dev.taru.android.ui.screens.sourcepicker.* --tests dev.taru.android.ui.screens.detail.* --tests dev.taru.android.ui.screens.player.* --tests dev.taru.android.ui.browse.BrowseSessionRouteStateTest --tests dev.taru.android.ui.artwork.TaruArtworkSlotsTest --no-daemon
apps/android/gradlew.bat -p apps/android :app:testDebugUnitTest --tests dev.taru.android.connection.* --tests dev.taru.android.browse.* --tests dev.taru.android.playback.* --tests dev.taru.android.userplayback.* --tests dev.taru.android.ui.browse.* --tests dev.taru.android.ui.connection.* --no-daemon
apps/android/gradlew.bat -p apps/android :app:testDebugUnitTest --no-daemon
```

Smoke gate when visible copy/navigation changes broadly:

```powershell
apps/android/scripts/Validate-AndroidLocal.ps1
```

Evidence:

- `apps/android/gradlew.bat -p apps/android :app:compileDebugUnitTestKotlin --no-daemon`
  — FAIL then PASS on 2026-05-21. Initial compile failed because the new
  detail copy regression missed an `assertFalse` import; imported it and reran
  successfully.
- `apps/android/gradlew.bat -p apps/android :app:testDebugUnitTest --tests dev.taru.android.ui.screens.settings.* --tests dev.taru.android.ui.screens.sourcepicker.* --tests dev.taru.android.ui.screens.detail.* --tests dev.taru.android.ui.screens.player.* --tests dev.taru.android.ui.browse.BrowseSessionRouteStateTest --tests dev.taru.android.ui.artwork.TaruArtworkSlotsTest --no-daemon`
  — PASS on 2026-05-21. Covers product-facing diagnostics labels, source
  picker version copy and accessibility labels, detail relationship copy,
  token-safe player session accessibility, API-gap copy, and artwork fallback
  labels.
- `apps/android/gradlew.bat -p apps/android :app:testDebugUnitTest --tests dev.taru.android.connection.* --tests dev.taru.android.browse.* --tests dev.taru.android.playback.* --tests dev.taru.android.userplayback.* --tests dev.taru.android.ui.browse.* --tests dev.taru.android.ui.connection.* --no-daemon`
  — PASS on 2026-05-21. Covers shared client user-message rewrites across
  connection, browse, playback, User Playback State, browse UI, and connection
  session tests.
- `apps/android/gradlew.bat -p apps/android :app:testDebugUnitTest --no-daemon`
  — PASS on 2026-05-21.
- `git diff --check` — PASS on 2026-05-21; only Git line-ending warnings were
  reported.

### AFCR-070 Architecture Reassessment

Focused gates:

```powershell
python -m json.tool docs/workstreams/android-fearless-client-refactor/WORKSTREAM.json > $null
git diff --check
```

Evidence:

- `HANDOFF.md` — updated on 2026-05-21 with the architecture reassessment
  note. The current closeout shape remains Kotlin package seams inside `:app`;
  generated Kotlin SDK, shared Rust/UniFFI client core, Gradle module split,
  artwork descriptors, broader Home/Library Detail paging, downloads/offline,
  external player handoff, and Android TV are split into separate target-state
  workstreams.
- `WORKSTREAM.json` — updated on 2026-05-21 so the continue policy points to
  AFCR-080 final verification and records the split/defer decisions.

## Final Closeout Gates

```powershell
apps/android/gradlew.bat -p apps/android :app:testDebugUnitTest --no-daemon
apps/android/gradlew.bat -p apps/android :app:assembleDebug --no-daemon
apps/android/scripts/Validate-AndroidLocal.ps1
git diff --check
```

Evidence:

- `apps/android/gradlew.bat -p apps/android :app:testDebugUnitTest --no-daemon`
  — PASS on 2026-05-21 after AFCR-070 and smoke harness copy updates.
- `apps/android/gradlew.bat -p apps/android :app:assembleDebug --no-daemon`
  — PASS on 2026-05-21 after AFCR-070 and smoke harness copy updates.
- `apps/android/scripts/Validate-AndroidLocal.ps1` — FAIL then PASS on
  2026-05-21. The first failures were stale smoke assertions expecting old
  developer-facing copy such as `Access Token`, `Authentication required`,
  `Public API`, `API backed`, `Related Media Items`, `Check source`,
  `Source / Version`, route-prepared labels, and `Server resume`. The smoke
  harness was updated to assert the AFCR-060 product language: server access
  key, sign-in required, From server, Related Titles, Check version, Version,
  ready playback labels, and Resume from server.
- Final passing local validation report:
  `apps/android/build/validation/20260521-032251/report.md`.
- Final passing smoke report:
  `apps/android/build/smoke-regression/20260521-032326/report.md`.
- `python -m json.tool docs/workstreams/android-fearless-client-refactor/WORKSTREAM.json > $null`
  — PASS on 2026-05-21 after closeout updates.
- `git diff --check` — PASS on 2026-05-21 after final gate updates; only Git
  line-ending warnings were reported.

## Evidence Log

| Date | Task | Command / Evidence | Result | Notes |
| --- | --- | --- | --- | --- |
| 2026-05-20 | AFCR-000 | Workstream docs created | Pending verification | Planning-only change. |
| 2026-05-20 | AFCR-010 | Introduced `PublicClientApiExecutor`; migrated connection, browse, playback, and User Playback State clients; removed duplicated route-local protocol helpers | PASS | Route clients now own route construction and category mapping only; executor owns transport failures, version headers, HTTP errors, public error envelopes, JSON decode failure, safe request previews, bearer redaction, and URL helper policy. |
| 2026-05-20 | AFCR-010 | `apps/android/gradlew.bat -p apps/android :app:compileDebugUnitTestKotlin --no-daemon` | PASS | Initial compile failed on missing `SafeRequestPreview` import in `TaruBrowseClient`; fixed and reran successfully. |
| 2026-05-20 | AFCR-010 | `apps/android/gradlew.bat -p apps/android :app:testDebugUnitTest --tests dev.taru.android.connection.* --no-daemon` | PASS | Connection health/auth probe behavior preserved. |
| 2026-05-20 | AFCR-010 | `apps/android/gradlew.bat -p apps/android :app:testDebugUnitTest --tests dev.taru.android.browse.* --no-daemon` | PASS | Browse route behavior, error mapping, and redaction preserved. |
| 2026-05-20 | AFCR-010 | `apps/android/gradlew.bat -p apps/android :app:testDebugUnitTest --tests dev.taru.android.playback.* --no-daemon` | PASS | Added coverage for session-preflight HTTP errors and API-version rejection via the shared executor. |
| 2026-05-20 | AFCR-010 | `apps/android/gradlew.bat -p apps/android :app:testDebugUnitTest --tests dev.taru.android.userplayback.* --no-daemon` | PASS | User Playback State routes preserved. |
| 2026-05-20 | AFCR-010 | `apps/android/gradlew.bat -p apps/android :app:testDebugUnitTest --no-daemon` | PASS | Full Android JVM unit suite passed after adapter migration. |
| 2026-05-20 | AFCR-010 | `git diff --check` | PASS | Only line-ending warnings from Git configuration were reported; no whitespace errors. |
| 2026-05-20 | AFCR-020 | Introduced `PlaybackRequestDescriptor`; migrated playback target, launch, player route, and tests away from route-level raw `TaruHttpRequest` with bearer headers | PASS | Authorization is built only from `authenticatedRequest(accessToken)` or the Media3 runtime boundary; descriptors reject Authorization headers and bearer-like header values. |
| 2026-05-20 | AFCR-020 | `apps/android/gradlew.bat -p apps/android :app:compileDebugUnitTestKotlin --no-daemon` | PASS | Production and unit-test Kotlin compile after token-safe playback launch migration. |
| 2026-05-20 | AFCR-020 | `apps/android/gradlew.bat -p apps/android :app:testDebugUnitTest --tests dev.taru.android.playback.* --tests dev.taru.android.player.* --tests dev.taru.android.ui.screens.player.* --no-daemon` | PASS | Playback/player focused suite validates descriptor construction, missing token behavior, safe diagnostics, runtime final-request auth, and exit effects. |
| 2026-05-20 | AFCR-020 | `apps/android/gradlew.bat -p apps/android :app:testDebugUnitTest --tests dev.taru.android.ui.browse.* --no-daemon` | PASS | Browse state tests validate Player routes remain transient in save payloads while start-playback behavior is preserved. |
| 2026-05-20 | AFCR-020 | `apps/android/gradlew.bat -p apps/android :app:testDebugUnitTest --no-daemon` | PASS | Full Android JVM unit suite passed after token-safe launch migration. |
| 2026-05-20 | AFCR-020 | `git diff --check` | PASS | Only line-ending warnings from Git configuration were reported; no whitespace errors. |
| 2026-05-21 | AFCR-030 | Extracted `BrowseSessionNavigation`, `BrowseRouteStatePolicy`, `BrowseRouteLoadingSession`, `BrowseItemDetailSession`, `BrowseSearchSession`, `BrowsePlaybackSession`, and `BrowseSessionStore` from broad `BrowseSession` orchestration | PASS | `BrowseSession` is now a composition shell; stale request IDs and route-family preparation live behind focused modules. |
| 2026-05-21 | AFCR-030 | `apps/android/gradlew.bat -p apps/android :app:compileDebugUnitTestKotlin --no-daemon` | PASS | Kotlin compile after browse module extraction and route-state tests. |
| 2026-05-21 | AFCR-030 | `apps/android/gradlew.bat -p apps/android :app:testDebugUnitTest --tests dev.taru.android.ui.browse.* --no-daemon` | PASS | Existing browse behavior preserved; new `BrowseSessionRouteStateTest` covers route preparation, unsupported facet API gap, stale request invalidation, and transient Player routes. |
| 2026-05-21 | AFCR-030 | `apps/android/gradlew.bat -p apps/android :app:testDebugUnitTest --no-daemon` | PASS | Full Android JVM unit suite passed after browse state deepening. |
| 2026-05-21 | AFCR-030 | `git diff --check` | PASS | Only line-ending warnings from Git configuration were reported; no whitespace errors. |
| 2026-05-21 | AFCR-040 | Added `ConnectionSecurityPolicy`; removed main manifest global cleartext; moved cleartext allowance to debug manifest; added final `JdkTaruHttpTransport` cleartext guard | PASS | Production defaults reject HTTP before token/transport; local-development policy explicitly allows HTTP. |
| 2026-05-21 | AFCR-040 | `apps/android/gradlew.bat -p apps/android :app:compileDebugUnitTestKotlin --no-daemon` | PASS | Kotlin compile after connection security policy changes. |
| 2026-05-21 | AFCR-040 | `apps/android/gradlew.bat -p apps/android :app:testDebugUnitTest --tests dev.taru.android.connection.* --no-daemon` | PASS | Connection tests cover cleartext rejection, local development opt-in, final transport guard, and safe diagnostics. |
| 2026-05-21 | AFCR-040 | `apps/android/gradlew.bat -p apps/android :app:testDebugUnitTest --no-daemon` | PASS | Full Android JVM unit suite passed after network security hardening. |
| 2026-05-21 | AFCR-040 | `git diff --check` | PASS | Only line-ending warnings from Git configuration were reported; no whitespace errors. |
| 2026-05-21 | AFCR-040 | Added debug/release `BuildConfig.TARU_ALLOW_CLEARTEXT_HTTP` wiring to `AndroidTaruAppEnvironmentFactory` | PASS | Debug/local builds explicitly use local-development cleartext policy; release defaults remain production-deny. |
| 2026-05-21 | AFCR-050 | Added shared paging helpers and load-more actions for Search, relationship indexes, and public-backed facets | PASS | Paging derives next offset from server-returned `PageInfo`; load-more failures stay attached to existing content rather than deleting already loaded rows. |
| 2026-05-21 | AFCR-050 | `apps/android/gradlew.bat -p apps/android :app:compileDebugUnitTestKotlin --no-daemon` | PASS | Kotlin compile after paging state, UI, data-source, and BuildConfig updates. |
| 2026-05-21 | AFCR-050 | `apps/android/gradlew.bat -p apps/android :app:testDebugUnitTest --tests dev.taru.android.ui.browse.* --no-daemon` | PASS | Browse UI/session tests cover Search append, relationship index append, facet append, no-more-page behavior, stale route protection, and existing route behavior. |
| 2026-05-21 | AFCR-050 | `apps/android/gradlew.bat -p apps/android :app:testDebugUnitTest --tests dev.taru.android.browse.* --tests dev.taru.android.connection.* --no-daemon` | PASS | Browse client paging query construction and connection security focused suites pass. |
| 2026-05-21 | AFCR-050 | `apps/android/gradlew.bat -p apps/android :app:testDebugUnitTest --no-daemon` | PASS | Full Android JVM unit suite passed after paging slice. |
| 2026-05-21 | AFCR-050 | `git diff --check` | PASS | Only line-ending warnings from Git configuration were reported; no whitespace errors. |
| 2026-05-21 | AFCR-060 | Rewrote user-facing copy across connection, settings, browse, detail, source picker, player, and safe client diagnostics; added `TaruStrings` and Android resources for stable common actions; added roles/labels for key custom controls | PASS | User-visible language now uses server compatibility, sign-in keys, titles, versions, watch progress, and server-backed lists instead of API gaps, token, route, source facts, and User Playback State terms. |
| 2026-05-21 | AFCR-060 | `apps/android/gradlew.bat -p apps/android :app:compileDebugUnitTestKotlin --no-daemon` | FAIL then PASS | Initial failure was missing `assertFalse` import in `MediaItemDetailRouteTest`; fixed and reran successfully. |
| 2026-05-21 | AFCR-060 | `apps/android/gradlew.bat -p apps/android :app:testDebugUnitTest --tests dev.taru.android.ui.screens.settings.* --tests dev.taru.android.ui.screens.sourcepicker.* --tests dev.taru.android.ui.screens.detail.* --tests dev.taru.android.ui.screens.player.* --tests dev.taru.android.ui.browse.BrowseSessionRouteStateTest --tests dev.taru.android.ui.artwork.TaruArtworkSlotsTest --no-daemon` | PASS | Focused product UI tests cover copy, API-gap language, version fallback labels, player session a11y redaction, and artwork fallback labels. |
| 2026-05-21 | AFCR-060 | `apps/android/gradlew.bat -p apps/android :app:testDebugUnitTest --tests dev.taru.android.connection.* --tests dev.taru.android.browse.* --tests dev.taru.android.playback.* --tests dev.taru.android.userplayback.* --tests dev.taru.android.ui.browse.* --tests dev.taru.android.ui.connection.* --no-daemon` | PASS | Focused client/message suites pass after shared diagnostics copy rewrites. |
| 2026-05-21 | AFCR-060 | `apps/android/gradlew.bat -p apps/android :app:testDebugUnitTest --no-daemon` | PASS | Full Android JVM unit suite passed after UI copy/a11y/localization seam work. |
| 2026-05-21 | AFCR-060 | `git diff --check` | PASS | Only line-ending warnings from Git configuration were reported; no whitespace errors. |
| 2026-05-21 | AFCR-070 | Architecture reassessment in `HANDOFF.md` and continue-policy update in `WORKSTREAM.json` | PASS | Kept package seams for closeout; split generated Kotlin SDK, shared Rust/UniFFI client core, Gradle module split, artwork descriptors, broader paging, downloads/offline, external player handoff, and Android TV into explicit follow-ons. |
| 2026-05-21 | AFCR-080 | `apps/android/gradlew.bat -p apps/android :app:testDebugUnitTest --no-daemon` | PASS | Full Android JVM unit suite passed after final smoke harness product-copy updates. |
| 2026-05-21 | AFCR-080 | `apps/android/gradlew.bat -p apps/android :app:assembleDebug --no-daemon` | PASS | Debug APK assemble passed after final smoke harness product-copy updates. |
| 2026-05-21 | AFCR-080 | `apps/android/scripts/Validate-AndroidLocal.ps1` | FAIL then PASS | Initial failures exposed stale smoke assertions for pre-AFCR-060 copy. Updated `Smoke-Emulator.ps1` to assert product language and reran successfully. Final report: `apps/android/build/validation/20260521-032251/report.md`; delegated smoke report: `apps/android/build/smoke-regression/20260521-032326/report.md`. |
| 2026-05-21 | AFCR-080 | `python -m json.tool docs/workstreams/android-fearless-client-refactor/WORKSTREAM.json > $null` | PASS | Workstream metadata remained valid JSON after closeout updates. |
| 2026-05-21 | AFCR-080 | `git diff --check` | PASS | Only line-ending warnings from Git configuration were reported; no whitespace errors. |
