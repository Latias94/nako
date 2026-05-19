# Android Developer Validation Entrypoint

Status: Closed
Last updated: 2026-05-19

## Why This Lane Exists

Android now has separate reliable pieces: Gradle unit/build gates,
single-state emulator smoke checks, and a stable smoke regression wrapper. The
remaining developer friction is command choice. A developer or agent should not
need to remember which Android command proves the local app is safe enough to
hand off after UI, playback, or smoke-harness work.

## Relevant Authority

- ADRs:
  - `docs/adr/0026-native-client-shells-with-shared-rust-client-core.md`
- Existing workstreams:
  - `docs/workstreams/android-client-foundation/`
  - `docs/workstreams/android-client-qa-harness/`
  - `docs/workstreams/android-material-expressive-ui/`
  - `docs/workstreams/android-smoke-regression-harness/`
- Android docs:
  - `apps/android/README.md`
  - `apps/android/SMOKE_FIXTURES.md`

## Problem

The Android confidence gate is currently discoverable only by reading several
documents. That is fragile during parallel development: one person may run unit
tests, another may run only `assembleDebug`, and another may run smoke but skip
the JVM tests that catch DTO/presentation regressions.

## Target State

When this lane closes:

- Android has one documented local developer validation command.
- The command composes Android JVM tests, optional debug build, and smoke
  regression without duplicating smoke navigation logic.
- The command writes a single report that links unit/build output and smoke
  evidence.
- Developers can run a faster non-smoke variant when no emulator is available.
- CI/device-farm packaging, golden screenshots, and structured JUnit/JSON
  exports remain follow-ons.

## In Scope

- A PowerShell validation entrypoint under `apps/android/scripts`.
- Android README and workstream documentation updates.
- A first local evidence run proving the default command shape.

## Out Of Scope

- CI device-farm integration.
- Golden visual diffing.
- New Android product features.
- New smoke fixture states.
- Rewriting the harness in Python.
- Moving Android into the Rust Cargo workspace.

## Architecture Direction

Keep this as a thin orchestration layer. `Smoke-Regression.ps1` remains the
owner of emulator state sequencing and failure categorization. The new
entrypoint should run Gradle gates, delegate smoke to the regression wrapper,
capture logs, and write a human-readable report under
`apps/android/build/validation/<timestamp>/`.

The default should optimize for handoff confidence, not fastest iteration.
Faster modes are explicit switches.

## Closeout

Closed on 2026-05-19. The local handoff command is:

```powershell
pwsh -NoProfile -File apps\android\scripts\Validate-AndroidLocal.ps1
```

The no-emulator variant is:

```powershell
pwsh -NoProfile -File apps\android\scripts\Validate-AndroidLocal.ps1 -SkipSmoke
```

Final evidence:

- No-emulator report:
  `apps/android/build/validation/20260519-094914/report.md`
- Default validation report:
  `apps/android/build/validation/20260519-095005/report.md`
- Delegated smoke regression report:
  `apps/android/build/smoke-regression/20260519-095037/report.md`

## Closeout Condition

This lane can close when:

- the validation command exists and is documented;
- a no-emulator mode passes locally;
- the default local validation command is run, or an exact environment blocker
  is recorded;
- `git diff --check` passes;
- remaining CI, visual, and report-format follow-ons are deferred or split.
