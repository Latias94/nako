# Android Client Foundation Handoff

Status: Proposed
Last updated: 2026-05-17

## Current State

The design baseline is documented. `ACF-010` created the Android scaffold
under `apps/android` as a single Gradle `:app` module outside the Rust Cargo
workspace. `ACF-020` added the first connection/auth slice. `ACF-030A` added
the first minimal browse tracer. `ACF-030B` added the first read-only Media
Item detail tracer.

Implemented scaffold:

- Gradle Wrapper under `apps/android`.
- Kotlin + Compose + Material 3 debug app shell.
- Dark-first Taru theme tokens for color, spacing, type, radius, poster and
  backdrop ratios, and touch target sizing.
- One local debug shell screen only.
- Local Android build README.

Implemented connection/auth slice:

- Compose setup shell with Server URL, Access Token, Test Connection, Save, and
  saved server profile switching.
- Direct Kotlin HTTP connection client for `GET /health` and a lightweight
  authenticated `/libraries?limit=1&offset=0` probe.
- Public Client API version inspection through `version` and
  `x-taru-api-version`.
- Public error-envelope parsing with sanitized diagnostics.
- Server profile repository with one active server and profile-scoped state.
- Android secure token vault; profiles store token references, not raw tokens.
- Mocked HTTP unit tests for success, unreachable, unauthorized, version
  mismatch, token redaction, and profile isolation.

Implemented browse tracer:

- Direct Kotlin browse client for public `GET /libraries` and
  `GET /items?limit=&offset=`.
- Minimal Android DTO mirrors for library and media item list responses.
- Root app shell switches from setup to Home/Libraries when an active server
  profile and token reference exist.
- Home/Libraries Compose shell with active-server-scoped loading, empty,
  unauthorized, unreachable, forbidden, public API error, invalid-response, and
  missing-token states.
- Mocked HTTP unit tests for browse pagination, empty-state input, sanitized
  diagnostics, active-server URL switching, and token-reference isolation.

Implemented item detail tracer:

- Direct Kotlin browse client method for public `GET /items/{item_id}`.
- Minimal Android DTO mirrors for the Public Client API `ItemDetailResponse`
  shape needed by the read-only detail screen.
- Home/Libraries item rows and posters open a read-only Media Item detail
  surface.
- Detail surface shows client-safe Canonical Metadata, metadata chips, and
  related response counts without Play/Resume controls.
- Mocked HTTP unit tests cover detail decode, safe request redaction,
  forbidden diagnostics, unsupported API version, invalid response, missing
  item, active-server URL switching, and token-reference isolation.

Resolved decisions:

- Android is the first implementation target.
- Android-first is implementation order, not product strategy.
- The first product slice is playback-first with a minimal media-library browse
  loop.
- Android uses native playback through Media3 ExoPlayer.
- Shared Rust client core may own protocol, auth, DTO, playback-decision, and
  request-construction logic, but not player instances.
- iOS remains a peer future native client target under ADR 0026.
- The first scaffold starts as one `:app` module. Split Gradle modules after
  connection, browse, and playback boundaries become real enough to justify
  the build overhead.
- ACF-020 starts with direct Kotlin HTTP. UniFFI is deferred until
  browse/search/playback request construction creates enough duplicated SDK
  logic to justify the packaging cost.
- The playback client visual baseline is Findroid-inspired and expressive-leaning, not an
  admin-console shell.

## Next Task

Continue with the next `ACF-030` sub-slice: Search shell and result
navigation, or Settings shell if server switching/re-authentication should be
hardened first.

Preserve the `ACF-020`, `ACF-030A`, and `ACF-030B` boundaries: consume Public
Client API DTOs from the active server profile, keep token values out of
diagnostics, and avoid Media3 playback, UniFFI, downloads, playback request
construction, Source / Version Picker behavior, Play/Resume activation, or
external-player work until their own tasks.

Recommended next task:

- `ACF-030C`: add a minimal Search shell using Public Client API search once
  the exact route query contract is confirmed, then navigate results to the
  existing read-only Media Item detail surface.

## Risks To Preserve

- Do not make Android-specific assumptions part of the Public Client API.
- Do not depend on AGPL server/internal crates from Android or shared client
  code.
- Do not create a Rust-owned player abstraction.
- Do not expand into server administration, metadata editing, addons, webhook,
  automation, or storage diagnostics in the first client slice.

## Validation Reminder

Use `EVIDENCE_AND_GATES.md` for commands. `ACF-010` validated
`apps/android/gradlew.bat :app:assembleDebug`. `ACF-020` validated
`apps/android/gradlew.bat :app:assembleDebug`,
`apps/android/gradlew.bat :app:testDebugUnitTest`,
`cargo check --workspace --tests`, and `git diff --check`. `ACF-030A`
validated `apps/android/gradlew.bat -p apps/android :app:testDebugUnitTest`
and `apps/android/gradlew.bat -p apps/android :app:assembleDebug`; final
closeout should also keep `cargo check --workspace --tests` and
`git diff --check` green. `ACF-030B` validated
`apps/android/gradlew.bat -p apps/android :app:testDebugUnitTest`; final
closeout should rerun the full Android/Rust gate set.
