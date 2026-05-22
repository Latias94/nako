# Android Client Follow-On Hardening

Status: Active
Last updated: 2026-05-22

## Why This Lane Exists

The Android client architecture-deepening lane closed with three intentional
follow-ons that are valuable before the next product wave: device/emulator smoke
evidence, token-vault modernization, and PlayerRuntime capability growth. This
lane turns those follow-ons into a durable execution plan with separate gates so
that smoke, secure storage, and player platform features do not blur into a
catch-all refactor.

## Relevant Authority

- `CONTEXT.md`
- `docs/adr/0026-native-client-shells-with-shared-rust-client-core.md`
- `docs/adr/0032-shared-rust-client-core-app-supplied-transport.md`
- `docs/workstreams/android-client-architecture-deepening/CLOSEOUT.md`
- `apps/android/README.md`

## Problem

1. The previous lane proved JVM tests, debug assemble, and local validation with
   smoke explicitly skipped. We still need concrete emulator or device smoke
   evidence for the current app shell.
2. `AndroidSecureTokenVault` is isolated behind `TokenVault`, but it still uses
   deprecated AndroidX Security `EncryptedSharedPreferences`. The storage seam
   needs a replacement or a safer migration wrapper before credentials become
   more product-critical.
3. PlayerRuntime is now a clean Android-owned seam, but platform capabilities
   such as MediaSession and Picture-in-Picture are not yet modeled behind it.

## Target State

- Android smoke validation has fresh evidence or an explicit environment-blocked
  record with enough diagnostics to reproduce.
- Token storage no longer depends on deprecated `EncryptedSharedPreferences` for
  new installs, or the lane records why a narrower migration follow-on is
  required after probing platform constraints.
- PlayerRuntime has a bounded capability slice behind Android-owned interfaces,
  starting with MediaSession/PiP only where safe and testable.
- Workstream docs record validation, residual risk, and follow-on routing.

## In Scope

- Android smoke scripts and smoke evidence under `apps/android/scripts` and
  generated validation output references.
- Android token-vault code under `apps/android/app/src/main/java/dev/taru/android/connection`.
- Android player runtime code under `apps/android/app/src/main/java/dev/taru/android/ui/screens/player` and `MainActivity`/manifest only when needed for platform integration.
- Android JVM and instrumentation/smoke tests where practical.
- Workstream docs under this directory.

## Out Of Scope

- No server API changes.
- No Rust-owned Android token storage, UI, Media3, MediaSession, PiP, Cast, or
  networking.
- No downloads/offline playback.
- No Android TV shell or external player handoff in this lane.
- No broad DataStore/Room persistence migration except what is strictly needed
  for token-vault migration.

## Architecture Direction

- Keep Android platform capabilities behind Android-owned seams.
- Token storage remains behind `TokenVault`; callers must not learn storage
  details or receive bearer tokens in diagnostics/UI.
- Player platform capabilities should attach to `PlaybackSessionRuntime` or a
  companion runtime capability object, not re-enter broad Composable route
  bodies.
- Smoke evidence should be reproducible and should distinguish PASS, SKIPPED,
  and BLOCKED states.

## Closeout Condition

This lane can close when:

- smoke evidence is recorded;
- token-vault migration is implemented or explicitly split with a proven blocker;
- the first PlayerRuntime capability slice is implemented or split with a proven
  platform/testability blocker;
- focused and broad Android validation gates pass or are explicitly skipped with
  reasons;
- `WORKSTREAM.json`, `TODO.md`, `EVIDENCE_AND_GATES.md`, and `HANDOFF.md` agree.

## Implemented Direction

- Smoke was run and recorded as DONE_WITH_CONCERNS because the local
  ADB/emulator environment became unstable. The scripts were hardened so the
  same evidence can be reproduced on a stable target.
- TokenVault now uses Android Keystore AES-GCM with hashed SharedPreferences
  keys and a no-deprecated migration-source seam.
- PlayerRuntime now owns platform session lifecycle, with Android framework
  MediaSession and a guarded Picture-in-Picture entrypoint as the first safe
  capability slice.
