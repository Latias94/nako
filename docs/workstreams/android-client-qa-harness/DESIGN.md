# Android Client QA Harness

Status: Completed
Last updated: 2026-05-18

## Why This Lane Exists

The Android Client Foundation and V2 Material 3 UI rewrite are now complete
enough for parallel development. The weak point is repeatable validation:
developers can run Gradle tests and manually use an emulator, but there is no
durable Android client harness that standardizes local smoke checks, screenshot
capture, fixture state, emulator assumptions, and evidence collection.

Without that harness, future Android UI, playback, and Public Client API work
will drift toward manual testing and screenshot folders that are useful in the
moment but hard for another agent or developer to reproduce.

## Relevant Authority

- `CONTEXT.md`
- `docs/adr/0026-native-client-shells-with-shared-rust-client-core.md`
- `docs/workstreams/android-client-foundation/`
- `docs/workstreams/android-material-expressive-ui/`
- `apps/android/README.md`

## Problem

- Android validation currently depends on separate Gradle commands plus manual
  emulator install, launch, navigation, and screenshot capture.
- Screenshot evidence is stored under untracked ad hoc directories such as
  `output/`, with no stable naming, cleanup, or closeout rule.
- There is no one-command local smoke path for "build, install, launch, capture
  known surfaces, and report evidence".
- UI work can accidentally regress setup, Home, Settings, detail, source
  picker, or player entry points without a cheap emulator-level signal.
- Test fixtures and seeded app state are not yet formalized, so emulator checks
  can depend on whatever state happened to be on the device.

## Target State

- A documented local Android QA harness exists for developers and agents.
- The harness can run focused JVM tests and debug assemble before emulator
  smoke checks.
- Emulator smoke checks have explicit prerequisites, device selection,
  install/launch behavior, screenshot output paths, and pass/fail criteria.
- The harness captures client-safe evidence without committing generated
  screenshots by default.
- Fixture/state handling is explicit enough for repeatable setup, Home,
  Settings, Server Profile, and error/empty-state checks.
- The lane records what remains manual and what can later move into CI.

## In Scope

- Scripts, Gradle tasks, README docs, or test utilities under `apps/android`
  that improve local Android client validation.
- Emulator smoke workflow for install, launch, basic navigation, and screenshot
  capture.
- Deterministic fixture/state strategy for client-safe visible surfaces.
- Documentation for evidence paths, cleanup, and what not to commit.
- Focused JVM tests or instrumentation scaffolding when they directly support
  the harness.

## Out Of Scope

- New product features such as downloads, PiP, external player, track picker,
  or User Playback State.
- Full visual-regression diff infrastructure or golden screenshot baselines
  that would require a separate maintenance policy.
- CI device farm integration unless a small local-compatible hook naturally
  falls out of the harness.
- Server API contract changes.
- V3 layout exploration.

## Starting Assumptions

| Assumption | Confidence | Evidence | Consequence if wrong |
| --- | --- | --- | --- |
| Local emulator smoke checks are the highest-value next testing step. | High | The V2 UI rewrite was manually sanity checked on an emulator. | Re-scope toward JVM-only tests if emulator access is unreliable. |
| Generated screenshots should remain untracked by default. | High | Current `output/` is untracked and useful as temporary evidence. | Add explicit tracked reference artifacts only after a review policy exists. |
| A script-first local harness is cheaper than full instrumentation coverage. | High | The app is early and still changing quickly. | Split a later instrumentation/golden workstream when UI stabilizes. |
| Public Client API boundaries must remain the data authority. | High | ADR 0026 and previous Android workstreams. | Harness fixtures must not rely on server internals or leaked locators. |

## Architecture Direction

Keep the harness close to the Android app but outside production UI logic:

- `apps/android/README.md` documents the standard local commands.
- `apps/android/scripts/` can own repeatable PowerShell scripts for build,
  install, launch, screenshot, and report generation.
- Gradle remains the source of truth for JVM tests and APK assembly.
- `adb` owns emulator install, launch, and screenshots.
- Generated evidence goes to ignored or untracked output paths unless a future
  workstream accepts golden/reference image policy.
- Fixture state should use public client surfaces or Android-local profile
  state only; it must not require AGPL server internals inside the Android app.

## Closeout Condition

This lane can close when:

- local Android QA commands are documented and reproducible;
- one-command or near-one-command smoke validation exists for an already
  running emulator;
- screenshot/evidence paths and cleanup rules are explicit;
- at least setup/Home/Settings/Server Profile smoke surfaces are covered or
  explicitly deferred with rationale;
- Android unit tests, debug assemble, and diff hygiene pass fresh;
- follow-ons for instrumentation, CI, or golden visual diffing are explicit.

## Closeout Result

Closed on 2026-05-18 after `ACQ-050`.

The local Android QA harness now documents and verifies:

- Android JVM tests and debug APK assembly;
- emulator install, launch, and report generation;
- deterministic `empty-setup` evidence for the setup surface;
- deterministic `profile-missing-token` evidence for Home, Settings, and
  Server Profile shell surfaces;
- named screenshots, UI hierarchy dumps, and pass/fail criteria files;
- untracked generated evidence under `apps/android/build/smoke/`.

Deferred follow-ons remain outside this lane: CI device execution, golden
visual diffing, server-backed demo data, detail/player smoke coverage, and
instrumentation test migration.
