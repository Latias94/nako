# Android Client Foundation Handoff

Status: Active
Last updated: 2026-05-18

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

Implemented Compose UI baseline rewrite:

- Replaced the tracer Home/Libraries tab shell with a Material 3 bottom
  navigation app shell and explicit routes for top-level destinations, Media
  Item Detail, Browse Facet placeholder, and Server Profile.
- Split browse UI into route/state, reusable components, Home, Libraries,
  Media Item Detail, Settings, placeholders, formatters, and preview files.
- Home and Libraries still use the existing Public Client API browse client and
  DTOs, including loading, empty, and sanitized failure states.
- Media Item Detail is now a playback-decision skeleton with disabled
  Play/Source placeholders, source summary, metadata chips, Cast & Crew, and
  relationship rows.
- Settings Home and Server Profile now use the existing server profile snapshot
  and token vault without displaying token values.
- `TaruAndroidAppContent` now passes the real profile snapshot into the browse
  shell and persists profile switches from Server Profile.
- Added Compose Material Icons Extended for standard Material iconography.

Implemented Search, Facet, and route hardening:

- Direct Kotlin browse client support for public `/search`, including query,
  comma-separated lightweight facets, pagination, and search hit decoding.
- Direct Kotlin browse client support for public related-item routes:
  `/genres/{genre_id}/items`, `/tags/{tag_id}/items`, and
  `/people/{person_id}/items`.
- Search top-level destination now uses a submitted-query screen backed by the
  active server profile and navigates results into Media Item Detail.
- Browse Facet Result now loads API-backed Genre, Tag, and Person targets when
  the UI has a stable facet id.
- Unsupported Library, Studio, Collection, Year, Item Kind, Source Mode, and
  missing-id relationship targets show explicit API-gap states rather than
  local pseudo filtering.
- Detail metadata chips map display labels to `ItemDetailResponse.genres` and
  `ItemDetailResponse.tags` relation ids by response order when available.
- Detail person relationships use `credits.person_id` only when present and do
  not invent cast or crew names from incomplete detail responses.

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
- The initial implementation baseline is v2 regular layout: Compose-friendly,
  Material 3 based, and expressive through restrained motion and artwork-muted
  accents rather than v3 irregular geometry.

## Next Task

Finish `ACF-030D` manual validation, then decide whether to close ACF-030 or
split the remaining API gaps into a Public Client API follow-up.

The visual direction is documented in `CLIENT_INTERFACE_DESIGN.md` and
reference screenshots live under
`docs/workstreams/android-client-foundation/reference-screenshots/`. ACF-030C
has translated that baseline into Compose code, and ACF-030D has hardened the
available public routes. The next step is not another visual rewrite.

Preserve the `ACF-020`, `ACF-030A`, `ACF-030B`, `ACF-030C`, and `ACF-030D`
boundaries: consume Public Client API DTOs from the active server profile, keep
token values out of diagnostics, and avoid Media3 playback, UniFFI, downloads,
playback request construction, Source / Version Picker behavior, Play/Resume
activation, or external-player work until their own tasks.

Recommended next task:

- Run and record the manual debug walkthrough against a running Taru server
  fixture and Android device/emulator:
  - connection/setup;
  - Home browse;
  - Libraries browse;
  - Media Item Detail;
  - Search result navigation;
  - Browse Facet Result for a real Genre/Tag/Person id if fixture data exists;
  - Settings;
  - Server Profile;
  - return to browse.
- Record whether the remaining API gaps should become a Public Client API
  workstream before `ACF-040`.

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
`apps/android/gradlew.bat -p apps/android :app:testDebugUnitTest`. `ACF-030C`
validated `apps/android/gradlew.bat -p apps/android :app:testDebugUnitTest`,
`apps/android/gradlew.bat -p apps/android :app:assembleDebug`, and
`git diff --check` on 2026-05-18. `ACF-030D` validated
`apps/android/gradlew.bat -p apps/android :app:testDebugUnitTest`,
`apps/android/gradlew.bat -p apps/android :app:assembleDebug`, and
`git diff --check` on 2026-05-18. Manual server/device walkthrough is still
pending and should be recorded before closing `ACF-030D`.
