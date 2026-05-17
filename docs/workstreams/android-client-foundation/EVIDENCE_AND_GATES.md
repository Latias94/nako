# Android Client Foundation Evidence And Gates

Status: Proposed
Last updated: 2026-05-17

This file defines validation expectations for the Android-first client lane.
Android scaffold commands are authoritative after `ACF-010`.

## Always-On Repository Gates

- `cargo fmt --all -- --check`
- `cargo check --workspace --tests` passed for `ACF-010` and `ACF-020` on
  2026-05-17.
- `cargo nextest run --workspace --no-fail-fast` when Rust/shared-client code
  changes warrant broad validation.
- `git diff --check` passed for `ACF-010` and `ACF-020` on 2026-05-17.

## Android Scaffold Gates

Validated for `ACF-010` on 2026-05-17:

- `apps/android/gradlew.bat :app:assembleDebug` passed.
- Root `Cargo.toml` workspace membership remains `members = ["crates/*"]`;
  `apps/android` is not a Rust workspace member.

Available follow-up commands:

- `apps/android/gradlew.bat :app:testDebugUnitTest` after Android unit tests
  exist.
- Android lint command selected when lint policy is added to the scaffold.

Linux/macOS equivalents may use `./gradlew`.

## Android Connection/Auth Gates

Validated for `ACF-020` on 2026-05-17:

- `apps/android/gradlew.bat :app:assembleDebug` passed.
- `apps/android/gradlew.bat :app:testDebugUnitTest` passed.
- Android unit tests cover:
  - successful `GET /health` preflight followed by an authenticated public
    route probe;
  - unreachable server diagnostics;
  - unauthorized access-token handling;
  - unsupported Public Client API version rejection;
  - invalid URL and missing access-token local validation;
  - active server switching and profile-scoped state isolation;
  - token vault reference behavior and safe request redaction.

Dependency boundary for `ACF-020`:

- Android uses direct Kotlin HTTP for setup/auth.
- Android does not depend on `taru-api`, `taru-server`, `taru-core`,
  `taru-streaming`, or `taru-transcode`.
- No Media3, UniFFI, Downloads, or external-player code is introduced.

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
