# Android Local Resume Smoke Evidence

Status: Closed
Last updated: 2026-05-19

## Why This Lane Exists

Android now persists device-local playback positions, but the emulator smoke
harness only proves server-backed browsing and playback launch. It does not yet
prove that a seeded local position is surfaced as local resume state in the UI.

This lane adds the smallest reliable smoke evidence for local resume without
claiming cross-device **User Playback State**.

## Relevant Authority

- `CONTEXT.md`: **User Playback State** is server-authoritative and distinct
  from client-local state.
- `docs/adr/0026-native-client-shells-with-shared-rust-client-core.md`
- `docs/workstreams/android-device-local-playback-position/`
- `docs/workstreams/android-smoke-regression-harness/`

## Problem

The debug server-backed smoke fixture can seed a profile and token, but it
cannot seed the device-local playback position store. As a result, the
`profile-with-media` smoke state exercises a first-play path only, leaving the
resume path unproven on a real emulator.

## Target State

When this lane closes:

- The debug-only smoke fixture can seed one device-local playback position for
  a concrete Media Item and Media Source.
- `profile-with-media` smoke evidence proves the local resume UI path with
  stable text assertions.
- Smoke evidence explicitly rejects cross-device or server-authoritative resume
  wording on the local resume surface.
- The implementation remains debug-only and does not alter Public Client API or
  server behavior.

## In Scope

- Debug-only Android smoke seed provider and request model.
- Android smoke PowerShell orchestration for resolving the fixture Media Item
  and Media Source ids.
- Focused Android tests for debug fixture request parsing.
- Smoke evidence and documentation updates.

## Out Of Scope

- Public Client API changes.
- Server-side playback progress reporting.
- Cross-device Continue Watching.
- Golden screenshot diffing.
- CI/device-farm execution.

## Architecture Direction

Keep the Android production path unchanged. The smoke fixture should seed
device-local resume through the same `SharedPreferencesDevicePlaybackPositionStore`
used by the app composition, but only from debug source sets and only when the
smoke harness provides a concrete item/source pair.

The smoke script should resolve item/source ids from the running fixture server
instead of hard-coding server implementation details. Assertions should prefer
product copy that already describes local-only behavior:

- `Resume on this device`
- `A device-local position exists for the selected source. Nako still checks the source before playback.`
- `Start resume`

The same surface must not use **User Playback State** or Continue Watching
language.

## Closeout Condition

This lane can close when:

- the debug seed provider can persist an optional local resume point;
- `profile-with-media` smoke resolves the fixture media/source ids and asserts
  the local resume path;
- focused Android tests pass;
- the focused smoke gate passes on the emulator;
- `git diff --check` passes;
- evidence and handoff docs name remaining follow-ons.

## Closeout

Closed on 2026-05-19. The debug-only smoke seed provider can now persist an
optional device-local playback position through
`SharedPreferencesDevicePlaybackPositionStore`. The `profile-with-media` smoke
state resolves the fixture Media Item and Media Source ids from the running
Public Client API server, seeds a local 0:01 resume point, and captures source
picker plus player evidence.

Final evidence:

- Focused debug fixture test:
  `apps\android\gradlew.bat -p apps\android :app:testDebugUnitTest --tests dev.nako.android.smoke.DebugSmokeFixtureSeedActivityTest --no-daemon`
- Focused smoke report:
  `apps/android/build/smoke/20260519-102517-profile-with-media-emulator-5554/report.md`
- Regression smoke report:
  `apps/android/build/smoke-regression/20260519-102943/report.md`
- Diff hygiene:
  `git diff --check`
