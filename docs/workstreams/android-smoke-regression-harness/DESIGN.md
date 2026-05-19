# Android Smoke Regression Harness

Status: Closed
Last updated: 2026-05-19

## Why This Lane Exists

Android client development now has a working app foundation, a local smoke
harness, and a server-backed `Night Harbor` fixture. The next risk is not raw
feature coverage; it is keeping parallel Android changes cheap to verify
without relying on memory, manual command sequencing, or stale emulator state.

## Relevant Authority

- ADRs:
  - `docs/adr/0025-openapi-public-client-sdk-contract.md`
  - `docs/adr/0026-native-client-shells-with-shared-rust-client-core.md`
- Existing docs:
  - `apps/android/README.md`
  - `apps/android/SMOKE_FIXTURES.md`
  - `CLIENT_INTERFACE_DESIGN.md`
- Related workstreams:
  - `docs/workstreams/android-client-foundation/`
  - `docs/workstreams/android-client-qa-harness/`
  - `docs/workstreams/android-server-backed-demo-fixtures/`
  - `docs/workstreams/android-material-expressive-ui/`

## Problem

`Smoke-Emulator.ps1` can validate individual Android fixture states, including
real server-backed media playback evidence, but there is no single local
regression entry point that runs the stable state set, captures a summary, and
sets clear expectations for developers before they hand work to another agent.

## Target State

When this lane closes:

- Android has one documented local regression command for the stable emulator
  smoke state set.
- The command builds once, runs selected fixture states, and writes a summary
  that points to each evidence directory.
- The stable default state set covers empty setup, missing-token profile shell,
  and server-backed media smoke.
- The harness keeps generated screenshots and server fixture artifacts local.
- Failure output is actionable enough to tell whether the break is build,
  device, fixture server, navigation, or surface criteria related.

## In Scope

- A local PowerShell wrapper under `apps/android/scripts`.
- README and smoke fixture documentation updates.
- Workstream evidence for the first full stable regression run.
- Small hardening to existing smoke scripts only when needed for reliable local
  regression execution.

## Out Of Scope

- CI device-farm integration.
- Golden visual diffing.
- Pixel-by-pixel screenshot assertions.
- New Android product features.
- New fake media data in Android.
- Authoritative cross-device User Playback State.
- HLS/remux/session depth beyond the current smoke state.
- UniFFI or shared Rust mobile-core packaging.

## Starting Assumptions

| Assumption | Confidence | Evidence | Consequence if wrong |
| --- | --- | --- | --- |
| `profile-with-media` is stable enough to be part of local regression. | High | `android-server-backed-demo-fixtures` closeout smoke passed. | Split server-backed smoke into an opt-in state until fixture reliability improves. |
| One build followed by `-SkipBuild` smoke runs is faster and less flaky than rebuilding per state. | High | Current smoke script builds by default and supports `-SkipBuild`. | The wrapper can fall back to per-state builds, but that should not be the default. |
| Generated evidence should stay under `apps/android/build/`. | High | Existing Android smoke docs and closeouts treat screenshots as generated local artifacts. | Add ignore rules or docs before any generated output leaks into commits. |
| A local emulator is available during this lane. | Medium | User said an emulator is open. | Keep commands documented and run what the local environment supports. |

## Architecture Direction

Keep `Smoke-Emulator.ps1` as the authoritative single-state harness. Add a thin
regression wrapper that composes stable fixture states instead of duplicating
ADB, navigation, or surface criteria logic. The wrapper should be boring:
resolve the Android root, build once unless asked to skip, call the existing
smoke script with explicit states, and write one summary report.

The Android app continues to consume only Public Client API route shapes for
server-backed media evidence. Debug-only profile seeding remains confined to
debug APK code and smoke scripts.

## Closeout

Closed on 2026-05-19. The stable local regression command is:

```powershell
pwsh -NoProfile -File apps\android\scripts\Smoke-Regression.ps1 -States empty-setup,profile-missing-token,profile-with-media
```

The final ASR-040 gate passed with all three states in one attempt:

- `apps/android/build/smoke-regression/20260519-093524/report.md`

`profile-with-media` now seeds its debug-only server profile through a
debug-only `ContentProvider.call` path instead of the seed Activity lifecycle.
This keeps smoke fixture setup out of the visible Activity stack and avoids
misclassifying emulator/system focus problems as product UI failures.

## Closeout Condition

This lane can close when:

- the regression wrapper exists and is documented;
- the stable default state set is explicit;
- the wrapper has been run locally, or an environment blocker is recorded with
  the exact failing command;
- Android unit/build gates still pass when script code changes warrant them;
- `git diff --check` passes;
- follow-ons for CI, golden screenshots, or deeper playback are split or
  deferred.
