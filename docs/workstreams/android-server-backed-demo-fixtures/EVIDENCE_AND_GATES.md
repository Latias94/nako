# Android Server-Backed Demo Fixtures — Evidence And Gates

Status: Closed
Last updated: 2026-05-18

## Smallest Current Repro

The current fixture provider repro is:

```powershell
pwsh -NoProfile -File apps\android\scripts\Smoke-Emulator.ps1 -FixtureState profile-with-media
```

This prepares the generated `Night Harbor` server fixture through real
`nako-server scan` and `import-nfo`, starts a short-lived local server, seeds a
debug-only Android Server Profile and encrypted token value, and captures Home,
detail, source picker, and player-safe launch evidence.

## Gate Set

### Targeted Planning Gate

```powershell
git diff --check
```

This proves the new workstream docs do not introduce whitespace errors.

### Android Unit Gate

```powershell
apps\android\gradlew.bat -p apps\android :app:testDebugUnitTest --no-daemon
```

This proves Android client fixture selection, profile seeding, request
construction, and presentation tests after Android code changes.

### Android Build Gate

```powershell
apps\android\gradlew.bat -p apps\android :app:assembleDebug --no-daemon
```

This proves the app still builds before emulator smoke runs.

### Media Smoke Gate

```powershell
pwsh -NoProfile -File apps\android\scripts\Smoke-Emulator.ps1 -FixtureState profile-with-media
```

This is the intended closeout smoke gate for Home, detail, source picker, and
player-safe launch evidence.

### Server/API Fixture Gate

The accepted provider command is:

```powershell
pwsh -NoProfile -File apps\android\scripts\Start-DemoFixtureServer.ps1 -PrepareOnly -SkipBuild
```

For request-level validation, start the generated config with
`nako-server --config apps/android/build/demo-fixtures/server-backed/nako.toml
serve`, then check:

- fixture responses use Public Client API route shapes;
- unsafe diagnostics and private paths are not exposed;
- playback decisions are enough for Android source picker and player-safe
  launch.

### Review Gate

Run `review-workstream` before accepting task or lane completion. Record
blocking findings, missing gates, and residual risks here or in `HANDOFF.md`.

### Verification Gate

Run `verify-rust-workstream` before marking the lane complete. Fresh evidence is
required for unit, build, fixture, smoke, and `git diff --check` gates relevant
to the shipped slice.

## Evidence Anchors

- `docs/workstreams/android-server-backed-demo-fixtures/DESIGN.md`
- `docs/workstreams/android-server-backed-demo-fixtures/TODO.md`
- `docs/workstreams/android-server-backed-demo-fixtures/MILESTONES.md`
- `docs/workstreams/android-server-backed-demo-fixtures/ROUTE_MATRIX.md`
- `apps/android/SMOKE_FIXTURES.md`
- `apps/android/scripts/Start-DemoFixtureServer.ps1`
- `apps/android/scripts/Smoke-Emulator.ps1`
- `apps/android/app/src/debug/java/dev/nako/android/smoke/DebugSmokeFixtureSeedActivity.kt`
- `apps/android/app/src/main/java/dev/nako/android/playback/PlaybackModels.kt`
- `crates/nako-api/src/openapi.rs`

## Current Evidence

2026-05-18:

- `ASD-010` completed as a planning and boundary freeze.
- No runtime code has changed yet.
- Media smoke was not runnable yet at this point because `profile-with-media`
  still needed a fixture provider.

2026-05-18:

- `ASD-020` completed as a route discovery and fixture strategy slice.
- `ROUTE_MATRIX.md` maps Android Home, Libraries, Search, detail, facet, source
  picker, and player-safe launch surfaces to existing Public Client API routes.
- The first provider strategy is a seeded local `nako-server` reached by
  Android through `adb reverse`; a public-route-compatible local test-server
  harness remains a fallback.
- Review note: first player smoke should prefer direct-play MP4. Android's
  stale `ClientTranscodePlan.inputLocator` requirement was fixed in `ASD-030`.

2026-05-18:

- `ASD-030` completed the first server-backed fixture provider.
- Added `apps/android/scripts/Start-DemoFixtureServer.ps1`.
- Added Android playback regression coverage for transcode decisions that omit
  internal `input_locator`.
- Removed Android's required `ClientTranscodePlan.inputLocator` field so the
  DTO matches the Public Client API.
- `pwsh -NoProfile -File apps\android\scripts\Start-DemoFixtureServer.ps1 -PrepareOnly -SkipBuild`
  passed: generated MP4/NFO fixture, scanned one source, probed one source, and
  imported one NFO item.
- Short-lived local server validation passed:
  - `GET /health` returned `ok`;
  - `GET /libraries?limit=50&offset=0` returned 1 library;
  - `GET /items?limit=24&offset=0` returned 1 item;
  - `GET /items/{item_id}` returned `Night Harbor` and 1 source;
  - `GET /sources/{source_id}/playback/decision` returned `direct_play`;
  - `HEAD /sources/{source_id}/stream` returned 200 and `video/mp4`;
  - combined public responses did not contain `input_locator`, `file:///`,
    `G:/`, `G:\`, `ffmpeg`, or `demo-fixture-token`.
- Focused Android gate passed:
  `apps\android\gradlew.bat -p apps\android :app:testDebugUnitTest --tests dev.nako.android.playback.NakoPlaybackClientTest --tests dev.nako.android.ui.screens.sourcepicker.SourcePickerDisplayModelTest --no-daemon`.
- Full Android unit gate passed:
  `apps\android\gradlew.bat -p apps\android :app:testDebugUnitTest --no-daemon`.
- Android debug build gate passed:
  `apps\android\gradlew.bat -p apps\android :app:assembleDebug --no-daemon`.
- `git diff --check` passed with existing Windows line-ending normalization
  warnings only.

2026-05-18:

- `ASD-040` completed the first Android media smoke state.
- Added a debug-only fixture seed activity under `apps/android/app/src/debug`
  that writes a safe Server Profile through `SharedPreferencesServerProfileStore`
  and the token value through `AndroidSecureTokenVault`.
- Added `profile-with-media` to `apps/android/scripts/Smoke-Emulator.ps1`.
  The script now prepares the server-backed fixture, starts a short-lived
  `nako-server`, applies `adb reverse`, seeds the debug profile/token, navigates
  Home -> detail -> source picker -> player, and removes the reverse mapping
  when it exits.
- Focused debug seed test passed:
  `apps\android\gradlew.bat -p apps\android :app:testDebugUnitTest --tests dev.nako.android.smoke.DebugSmokeFixtureSeedActivityTest --no-daemon`.
- Media smoke gate passed:
  `pwsh -NoProfile -File apps\android\scripts\Smoke-Emulator.ps1 -FixtureState profile-with-media`.
- Smoke evidence was generated at
  `apps/android/build/smoke/20260518-235008-profile-with-media-emulator-5554/`
  with PASS criteria for `home`, `detail`, `source-picker`, and `player`.

2026-05-18 closeout:

- Workstream review found no blocking findings for the shipped scope.
- Verification gates passed:
  - `apps\android\gradlew.bat -p apps\android :app:testDebugUnitTest --no-daemon`;
  - `apps\android\gradlew.bat -p apps\android :app:assembleDebug --no-daemon`;
  - `pwsh -NoProfile -File apps\android\scripts\Start-DemoFixtureServer.ps1 -PrepareOnly -SkipBuild`;
  - `cargo check -p nako-server`;
  - `cargo build -p nako-server -vv`;
  - `pwsh -NoProfile -File apps\android\scripts\Smoke-Emulator.ps1 -FixtureState profile-with-media`;
  - `git diff --check`.
- Cleanup checks passed: no `nako-server` process remained after smoke, and
  `adb reverse --list` had no residual reverse mapping.
- The first attempt at the full smoke gate hit a transient `cargo build
  -p nako-server` failure after warnings only; immediate focused `cargo check`
  and `cargo build -p nako-server -vv` passed, and the full smoke gate passed
  on rerun.

## Notes

Do not list generated screenshots or reports as committed artifacts by default.
Evidence output under `apps/android/build/smoke/` remains generated local
evidence unless a future golden/reference image policy accepts tracked files.
