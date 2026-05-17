# Android Client Foundation Milestones

Status: Proposed
Last updated: 2026-05-17

## ACF-M0: Scope And Architecture Baseline

Status: complete

Exit criteria:

- Android-first is recorded as implementation order, not product strategy.
- Playback-first with minimal media-library browse loop is recorded.
- Rust shared core and native Android player ownership are separated.
- ADR 0026 is referenced as the architecture authority.

Evidence:

- `DESIGN.md`
- `TODO.md`
- `WORKSTREAM.json`

## ACF-M1: Android Scaffold

Status: complete

Exit criteria:

- `apps/android` exists.
- The app builds a minimal debug shell.
- The first theme follows the playback client visual baseline: immersive, artwork-led,
  dark-first, playback-confident, source-clear, expressive-leaning, and
  Material 3 based.
- Android files are not part of the Rust Cargo workspace.
- Local build docs exist.

Validation:

- `apps/android/gradlew.bat :app:assembleDebug` passed on 2026-05-17.
- `cargo check --workspace --tests` passed on 2026-05-17.

## ACF-M2: Public Client Connection

Status: complete

Exit criteria:

- Android can configure a Taru base URL and bearer token.
- Android presents the credential as an access token and stores it securely.
- Token values are redacted from logs, diagnostics, screenshots, and safe
  request previews.
- Android can store multiple server profiles while keeping one active server.
- Browse, search, playback, cache, and future download state are scoped by
  active server profile.
- Health preflight and API version handling work.
- Public error envelope handling has tests.
- Setup/auth errors are actionable and sanitized.
- Client dependencies stay outside AGPL server/internal crates.

Candidate validation:

- `apps/android/gradlew.bat :app:assembleDebug` passed on 2026-05-17.
- `apps/android/gradlew.bat :app:testDebugUnitTest` passed on 2026-05-17.
- `cargo check --workspace --tests` passed on 2026-05-17.
- `git diff --check` passed on 2026-05-17.
- No shared Rust client crate was introduced in ACF-020; Android remains
  outside the Rust Cargo workspace.

## ACF-M3: Browse-To-Item Loop

Status: in_progress

Exit criteria:

- Android can list libraries.
- Android can list and open media items.
- Phone and tablet share one touch-first route model with responsive layouts.
- Home works as a playback launchpad even before resume/latest API support is
  complete.
- Resume and Continue Watching UI uses authoritative Public Client API state
  when available and does not promote device-local transient state to
  cross-device facts.
- Search supports global keyword discovery and safe result navigation without
  client-only advanced filter semantics.
- Settings cover client identity, connection, theme, and basic playback
  preferences without becoming server administration.
- Media Item Detail prioritizes Play/Resume, hierarchy navigation, and
  explainable playback state.
- Empty, loading, and public error states are represented.
- Browse/search/detail errors offer useful recovery actions and sanitized
  diagnostics.
- The flow remains based on Public Client API DTOs.

Candidate validation:

- Android unit tests with mocked public API responses.
- Manual local debug app walkthrough.

Progress:

- `ACF-030A` completed on 2026-05-17:
  - active-server-scoped `GET /libraries` browse client;
  - minimal `GET /items?limit=&offset=` tracer;
  - Home/Libraries Compose shell;
  - loading, empty, unauthorized, unreachable, and public error states;
  - mocked API tests for pagination, empty input, diagnostics, active-server
    switching, and token redaction.
- `ACF-030B` completed on 2026-05-17:
  - active-server-scoped `GET /items/{item_id}` detail client;
  - read-only Media Item detail surface from Home/Libraries item lists;
  - client-safe Canonical Metadata, response counts, and detail error states;
  - mocked API tests for detail decode, diagnostics, invalid response,
    unsupported API version, active-server switching, and token redaction.

Remaining before milestone completion:

- search shell;
- settings shell;
- manual walkthrough from connection to item detail.

## ACF-M4: Playback Decision Loop

Status: pending

Exit criteria:

- Android can ask Taru for playback decisions.
- Direct, remux, and HLS request construction is represented.
- Source / Version Picker appears when multiple playable sources or variants
  exist.
- The picker shows only client-safe source facts and playback-mode
  consequences.
- Tokens and sensitive headers are not logged.
- Playback errors are mapped to user-facing categories.
- Playback decision errors offer useful recovery actions and sanitized
  diagnostics.

Candidate validation:

- Request-construction tests.
- Local server smoke test for at least one source.

## ACF-M5: Media3 Playback Smoke

Status: pending

Exit criteria:

- Android plays at least one Taru public playback route through Media3.
- Player lifecycle is tied to Android lifecycle.
- Basic playback controls, loading/buffering state, seek behavior,
  full-screen/orientation behavior, exit behavior, and error states exist.
- HLS/remux/transcode session cancellation on exit is handled when the public
  session route supports it.
- Playback errors offer useful recovery actions and sanitized diagnostics.
- Follow-up gaps for progress/resume, subtitles, track selection, offline, and
  cast are recorded.
- Public Client API gaps for authoritative User Playback State are recorded
  before full Resume/Continue Watching behavior is claimed complete.
- External player handoff remains deferred unless a secure short-lived handoff
  API is accepted.
- Offline/Downloads remains deferred but recorded as a second-phase core client
  capability requiring separate lifecycle and storage design.

Candidate validation:

- Manual device or emulator playback smoke test.
- Instrumented test plan if automated playback is not practical in CI.
