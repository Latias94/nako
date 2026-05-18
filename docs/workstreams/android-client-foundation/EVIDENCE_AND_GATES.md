# Android Client Foundation Evidence And Gates

Status: Proposed
Last updated: 2026-05-18

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

## Android Browse Tracer Gates

Validated for `ACF-030A` on 2026-05-17:

- `apps/android/gradlew.bat -p apps/android :app:testDebugUnitTest` passed.
- `apps/android/gradlew.bat -p apps/android :app:assembleDebug` passed.
- `cargo fmt --all -- --check` passed.
- `cargo check --workspace --tests` passed.
- `git diff --check` passed with line-ending normalization warnings only.
- Android unit tests cover:
  - paginated `GET /libraries`;
  - minimal paginated `GET /items`;
  - empty library response as successful empty-state input;
  - unauthorized browse diagnostics;
  - unreachable browse diagnostics;
  - public error-envelope redaction for tokens, local paths, and FFmpeg
    commands;
  - active profile switching changing request base URL and token reference.

Dependency boundary for `ACF-030A`:

- Android browse uses direct Kotlin HTTP against Public Client API routes.
- Android DTOs mirror only the public protocol fields needed by the first
  browse shell.
- Android still does not depend on `taru-api`, `taru-server`, `taru-core`,
  `taru-streaming`, or `taru-transcode`.
- No Media3, UniFFI, Downloads, external-player, playback decision, or
  playback request construction code is introduced.

Remaining validation before full `ACF-030` can close:

- mocked tests for item detail and search navigation;
- manual debug app walkthrough from server connection to item detail.

## Android Item Detail Tracer Gates

Validated for `ACF-030B` on 2026-05-17:

- `apps/android/gradlew.bat -p apps/android :app:testDebugUnitTest` passed.
- `apps/android/gradlew.bat -p apps/android :app:assembleDebug` passed.
- `cargo fmt --all -- --check` passed.
- `cargo check --workspace --tests` passed.
- `git diff --check` passed with line-ending normalization warnings only.
- Android unit tests cover:
  - successful `GET /items/{item_id}` detail decode;
  - safe detail request redaction;
  - forbidden detail diagnostics;
  - unsupported Public Client API version rejection on detail response;
  - invalid detail response diagnostics;
  - missing item local validation;
  - active profile switching changing detail request base URL and token
    reference.

Dependency boundary for `ACF-030B`:

- Android detail uses direct Kotlin HTTP against Public Client API routes.
- Android DTOs mirror only public protocol fields needed by the first read-only
  detail shell.
- Android still does not depend on `taru-api`, `taru-server`, `taru-core`,
  `taru-streaming`, or `taru-transcode`.
- No Media3, Play/Resume activation, Source / Version Picker, UniFFI,
  Downloads, external-player, playback decision, or playback request
  construction code is introduced.

Remaining validation before full `ACF-030` can close:

- mocked tests for search navigation;
- manual debug app walkthrough from server connection to item detail against a
  running Taru server fixture.

## Android Compose UI Baseline Rewrite Gates

Validated for `ACF-030C` on 2026-05-18:

- `apps/android/gradlew.bat -p apps/android :app:testDebugUnitTest` passed.
- `apps/android/gradlew.bat -p apps/android :app:assembleDebug` passed.
- `git diff --check` passed with line-ending normalization warnings only.

What this proves:

- Existing connection, token, server-profile, browse pagination, detail decode,
  safe request redaction, sanitized diagnostics, active-server switching, and
  token-reference isolation tests still pass after the Compose rewrite.
- The Android debug APK builds with the new Material 3 browse shell, split
  screen/component structure, and Material Icons Extended dependency.
- The rewritten UI keeps Android outside the Rust Cargo workspace and does not
  introduce Media3, UniFFI, playback decisions, downloads, external-player
  handoff, or server/internal Rust crate dependencies.

Changed Android UI/build scope:

- `apps/android/app/src/main/java/dev/taru/android/ui/browse/TaruBrowseShell.kt`
- `apps/android/app/src/main/java/dev/taru/android/ui/browse/BrowseComponents.kt`
- `apps/android/app/src/main/java/dev/taru/android/ui/browse/BrowseFormatters.kt`
- `apps/android/app/src/main/java/dev/taru/android/ui/browse/BrowseModels.kt`
- `apps/android/app/src/main/java/dev/taru/android/ui/browse/HomeScreen.kt`
- `apps/android/app/src/main/java/dev/taru/android/ui/browse/LibrariesScreen.kt`
- `apps/android/app/src/main/java/dev/taru/android/ui/browse/MediaItemDetailScreen.kt`
- `apps/android/app/src/main/java/dev/taru/android/ui/browse/PlaceholderScreens.kt`
- `apps/android/app/src/main/java/dev/taru/android/ui/browse/SettingsScreens.kt`
- `apps/android/app/src/main/java/dev/taru/android/ui/browse/TaruBrowseShellPreview.kt`
- `apps/android/app/src/main/java/dev/taru/android/ui/TaruAndroidApp.kt`
- `apps/android/app/build.gradle.kts`
- `apps/android/gradle/libs.versions.toml`

Gates not run:

- Manual debug walkthrough against a running Taru server fixture was not run in
  this session because no fixture/device walkthrough was started. Track this in
  `ACF-030D`.
- Rust `cargo fmt`, `cargo check`, and `cargo nextest` were not rerun for
  `ACF-030C` because this change touched Android Kotlin/Compose, Gradle, and
  workstream documentation only; no Rust crate code or Cargo manifests changed.

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
