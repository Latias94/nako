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

## Android Search, Facet, And Walkthrough Hardening Gates

Validated for `ACF-030D` on 2026-05-18:

- `cargo fmt --all -- --check` passed.
- `cargo nextest run -p taru-server http::tests::catalog --no-fail-fast`
  passed: 3 catalog HTTP tests passed, including
  `search_route_returns_indexed_items` with `limit=12&offset=0`.
- `cargo build -p taru-server` passed with pre-existing unused-code warnings
  in server runtime/config scaffolding.
- `apps/android/gradlew.bat -p apps/android :app:testDebugUnitTest` passed.
- `apps/android/gradlew.bat -p apps/android :app:assembleDebug` passed.
- `git diff --check` passed with Windows line-ending normalization warnings
  only.

What this proves:

- The Android browse client encodes and decodes the public `/search` route,
  including query, comma-separated facet string, pagination, result hits, and
  safe request redaction.
- The Android browse client encodes and decodes public Genre, Tag, and Person
  related-item routes:
  - `GET /genres/{genre_id}/items`;
  - `GET /tags/{tag_id}/items`;
  - `GET /people/{person_id}/items`.
- Search is no longer a placeholder destination; submitted searches load
  active-server-scoped results and navigate to Media Item Detail.
- Browse Facet Result is no longer a placeholder route for supported targets;
  it loads API-backed Genre, Tag, and Person targets when a stable id is
  available.
- Unsupported facet targets remain explicit API-gap pages instead of
  client-only pseudo filters.
- Existing connection, token, server-profile, browse pagination, detail decode,
  sanitized diagnostics, active-server switching, and token-reference isolation
  tests still pass after the route hardening.
- The public `/search?q=&limit=&offset=` route accepts numeric pagination
  query values from real HTTP clients. The Android walkthrough exposed a
  server-side query deserialization bug when `/search?q=Harbor&limit=24&offset=0`
  returned `invalid type: string "24", expected u32`; the server fix removes
  flattened pagination from `SearchPageQuery` and adds test coverage for the
  route query shape.

Manual real-server walkthrough evidence on 2026-05-18:

- Device/emulator: `Pixel_3a_API_34_extension_level_7_x86_64`
  (`emulator-5554`).
- Server: local `target\debug\taru-server.exe --config
  tmp/android_real_server_fixture\taru.toml serve` on `127.0.0.1:3018`, reached
  by Android through `adb reverse tcp:3018 tcp:3018`.
- Fixture: one Movies library item, `Night Harbor`, imported through scan and
  NFO import with Genre `Mystery`, Tag `Lighthouse`, one credit, and one media
  source candidate.
- Verified HTTP route: `/search?q=Harbor&limit=24&offset=0` returned 200 with
  `Night Harbor` and page `{ limit: 24, offset: 0, returned: 1 }`.
- Verified UI flow: connection/setup to `http://127.0.0.1:3018`, Home,
  Libraries, Media Item Detail, Search query `Harbor`, Search result
  `Night Harbor` with `100%` score, Genre Browse Facet Result for `Mystery`
  showing `Related Media Items` and `Night Harbor`, Settings, Server Profile,
  and return to Home.
- Verified token safety: Settings and Server Profile showed secure-token copy
  such as `Stored securely on this device` and `Sanitized report`; the
  walkthrough UI dumps did not show the raw walkthrough token.

API gaps intentionally not implemented in Android under `ACF-030D`:

- Library-scoped item browsing and library facets.
- Studio related-item route.
- Collection related-item route.
- Year related-item route.
- Media Item kind related-item route.
- Rich credit/person display data in `ItemDetailResponse` for cast and crew
  names.

Gates not run:

- Full `cargo nextest run --workspace --no-fail-fast` was not rerun because the
  Rust change is scoped to the `taru-server` `/search` HTTP query parser and
  catalog route test. The targeted `taru-server` catalog HTTP suite plus
  `cargo build -p taru-server` cover the changed server behavior.

## Android Playback Decision And Request Construction Gates

Validated for `ACF-040` on 2026-05-18:

- `apps/android/gradlew.bat -p apps/android :app:testDebugUnitTest --tests dev.taru.android.playback.TaruPlaybackClientTest`
  passed.
- `apps/android/gradlew.bat -p apps/android :app:testDebugUnitTest` passed.
- `apps/android/gradlew.bat -p apps/android :app:assembleDebug` passed.
- `git diff --check` passed with Windows line-ending normalization warnings
  only.

What this proves:

- The Android playback client encodes and decodes the public
  `/sources/{source_id}/playback/decision` route, including client capability
  query parameters and Public Client API version checks.
- Android constructs direct stream, direct HEAD preflight, remux stream, HLS
  playlist, and HLS segment request targets using stable Public Client API
  paths, methods, query strings, range headers, and bearer auth headers.
- Safe request previews redact bearer tokens and do not expose access-token
  values.
- Playback diagnostics sanitize bearer tokens, local filesystem paths, file
  URLs, and FFmpeg command text from public error envelopes and transport
  errors.
- Media Item Detail now lets the user choose a Media Source, request a playback
  decision, and inspect a client-safe prepared route summary without starting
  Media3 playback.
- The real app can request a playback decision from a real local Taru server
  fixture and render the server-selected HLS route as a client-safe preview.

Manual real-server walkthrough evidence on 2026-05-18:

- Device/emulator: `Pixel_3a_API_34_extension_level_7_x86_64`
  (`emulator-5554`).
- Server: local `target\debug\taru-server.exe --config
  tmp/android_real_server_fixture\taru.toml serve` on `127.0.0.1:3018`, reached
  by Android through `adb reverse tcp:3018 tcp:3018`.
- Fixture: one Movies library item, `Night Harbor`, with one media source
  candidate `Night Harbor.mkv`.
- Verified UI flow: setup save, Home, `Night Harbor` Media Item Detail,
  `Playback Source Selection`, source candidate `Night Harbor.mkv`,
  `Request decision`, `HLS route prepared`, decision reason
  `client does not advertise support for mkv container`, safe route preview
  `GET http://127.0.0.1:3018/sources/.../stream/hls/playlist.m3u8`, and
  `Playback starts in ACF-050.`
- Verified token and local-path safety: UI dump checks found the expected
  playback decision text and did not find `walkthrough-token`, `file:///`,
  `G:/`, or `ffmpeg`.

Gates not run:

- Rust gates were not rerun for ACF-040 because this slice changed Android
  Kotlin/Compose and workstream documentation only; no Rust crate code or
  Cargo manifests changed.

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

## Android Media3 Playback Smoke Gates

Validated for `ACF-050` on 2026-05-18:

- `apps/android/gradlew.bat -p apps/android :app:compileDebugKotlin --no-daemon --rerun-tasks`
  passed.
- `apps/android/gradlew.bat -p apps/android :app:testDebugUnitTest --tests dev.taru.android.player.PlaybackLaunchTest --no-daemon`
  passed.
- `apps/android/gradlew.bat -p apps/android :app:testDebugUnitTest --no-daemon`
  passed.
- `apps/android/gradlew.bat -p apps/android :app:assembleDebug --no-daemon`
  passed.

What this proves:

- Android builds with Media3 ExoPlayer, HLS, and UI dependencies.
- The player route can consume the `ACF-040` prepared playback target and pass
  required HTTP headers to Media3 without exposing bearer token values through
  launch debug output.
- Existing Android connection, browse, search, detail, playback-decision, and
  request-construction unit tests still pass with the Media3 smoke slice.

Manual real-server playback smoke evidence on 2026-05-18:

- Device/emulator: `Pixel_3a_API_34_extension_level_7_x86_64`
  (`emulator-5554`), Android API 34.
- Server: local `target\debug\taru-server.exe --config
  tmp/android_real_server_fixture\taru.toml serve` on `127.0.0.1:3018`, reached
  by Android through `adb reverse tcp:3018 tcp:3018`.
- Fixture media: local `Night Harbor.mkv`, regenerated as a 2 second H.264/AAC
  Matroska file and re-probed with `target\debug\taru-server.exe --config
  tmp/android_real_server_fixture\taru.toml scan-all`.
- Playback decision path: Media Item Detail -> `Request decision` ->
  `Remux route prepared` for
  `/sources/{source_id}/stream/remux?output_container=mp4` -> `Start playback`.
- Observed Media3 UI state: PlayerView route opened; controller showed
  `00:02 / 00:02`; app status text showed `Media3: Ended`.
- Observed logcat state: ExoPlayer initialized as AndroidX Media3 `1.10.1`;
  H.264 video decoder and AAC audio decoder were created; no playback error
  appeared in the filtered logcat output.
- Verified token safety: player UI did not show the raw walkthrough token.

Gates not run:

- Android instrumented tests were not added for ACF-050 because the target
  claim is a real emulator/server playback smoke and local CI does not yet
  have a stable Media3 instrumentation harness.
- Rust gates were not rerun for ACF-050 because this slice changed Android
  Kotlin/Compose, Gradle dependencies, and workstream documentation only; no
  Rust crate code or Cargo manifests changed.

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
