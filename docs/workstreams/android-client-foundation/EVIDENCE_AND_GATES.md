# Android Client Foundation Evidence And Gates

Status: Proposed
Last updated: 2026-05-17

This file defines validation expectations for the Android-first client lane.
Commands are candidates until the Android project scaffold exists.

## Always-On Repository Gates

- `cargo fmt --all -- --check`
- `cargo check --workspace --tests`
- `cargo nextest run --workspace --no-fail-fast` when Rust/shared-client code
  changes warrant broad validation.
- `git diff --check`

## Android Scaffold Gates

Candidate commands after `apps/android` exists:

- `apps/android/gradlew.bat :app:assembleDebug`
- `apps/android/gradlew.bat testDebugUnitTest`
- Android lint command selected by the Gradle scaffold.

Linux/macOS equivalents may use `./gradlew`.

## Shared Rust Client-Core Gates

If a mobile shared Rust crate is introduced:

- `cargo check -p <mobile-client-core-crate> --tests`
- `cargo nextest run -p <mobile-client-core-crate> --no-fail-fast`
- dependency-tree or manifest checks proving no dependency on:
  - `taru-api`
  - `taru-server`
  - `taru-core`
  - `taru-streaming`
  - `taru-transcode`

If UniFFI is introduced:

- binding generation command documented in the Android README;
- generated binding drift check, if generated bindings are committed;
- Android build proves the native library is packaged for the selected ABIs.

## Public API Contract Gates

- Mocked Android or shared-core tests for:
  - health preflight;
  - bearer-token use;
  - unsupported API version;
  - public error envelope parsing;
  - sanitized diagnostics that exclude token values, secret references, local
    filesystem paths, FFmpeg commands, raw provider payloads, and server-local
    output paths;
  - pagination;
  - playback decision response handling;
  - direct/remux/HLS request construction;
  - active-server scoping for any device-local transient playback position.

## Playback Smoke Evidence

The first playback evidence should record:

- Taru server command/config used for the smoke test;
- media source type: local or remote;
- playback route: direct, remux, or HLS;
- Android device/emulator model and OS version;
- observed Media3 state transitions;
- whether seek, pause/resume, and error teardown worked;
- known gaps for subtitles, audio tracks, progress, PiP, and cast.

## Documentation Evidence

Closeout should update:

- `README.md`
- `DESIGN.md`
- `TODO.md`
- `MILESTONES.md`
- `HANDOFF.md`
- `docs/workstreams/README.md`
- `docs/ROADMAP.md` or `docs/GOALS.md` only when this lane becomes an active
  numbered implementation goal.
