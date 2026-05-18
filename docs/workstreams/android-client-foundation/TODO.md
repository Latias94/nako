# Android Client Foundation TODO

Status: Proposed
Last updated: 2026-05-18

## Task Ledger

### ACF-010: Android Scaffold And Repository Boundary

Status: complete
Owner: codex

Create the initial Android project under `apps/android` with a minimal Kotlin
and Compose app shell. Keep it outside the Rust Cargo workspace.

Scope:

- Android Gradle project scaffold.
- App entry point and one local debug screen.
- Basic dark-first Compose Material 3 theme aligned with the playback client visual baseline
  in `UX_CONTEXT.md`, including initial color roles, spacing, type roles,
  poster aspect ratios, corner radius, touch targets, and component semantics.
- README with local Android prerequisites and build command.
- No Taru server calls yet.

Validation:

- Android debug build command is documented and passes on a configured Android
  toolchain: `apps/android/gradlew.bat :app:assembleDebug`.
- Existing Rust workspace checks still ignore Android app files unless
  explicitly requested: root `Cargo.toml` workspace membership remains
  `members = ["crates/*"]`.
- The scaffold remains local-only: no server connection, access token,
  Public Client API, Media3, UniFFI, Downloads, or external-player code.

### ACF-020: Public Client Connection And Auth Slice

Status: complete
Owner: codex
Depends on: ACF-010

Add base URL and bearer-token configuration, health preflight, API-version
inspection, and public error-envelope handling.

Scope:

- Local connection state.
- Multiple server profile model with one active server at a time.
- Secure token reference per server profile.
- Active-server scoping for all browse, search, playback, cache, and future
  download state.
- Server URL plus access-token setup flow with Test Connection and Save.
- Token redaction for logs, diagnostics, screenshots, and safe request output.
- `GET /health` preflight.
- Unauthorized and unsupported-version UX.
- Actionable setup/auth errors for invalid URL, unreachable server,
  unauthorized token, unsupported API version, and TLS/certificate failures.
- Decision on direct Kotlin HTTP vs Rust/UniFFI client bridge for the first
  slice.

Validation:

- Mocked HTTP tests cover success, unreachable server, unauthorized, and
  version mismatch.
- Tests prove switching active server changes request base URL and does not
  mix profile-scoped state.
- Tests prove token values are hidden and never included in diagnostics or
  safe request previews.
- Tests cover sanitized diagnostics for representative public error envelopes.
- `apps/android` stores access-token values behind token references; server
  profile records do not carry raw token values.
- ACF-020 records direct Kotlin HTTP as the first Android connection strategy;
  UniFFI remains deferred until browse/search/playback request duplication is
  large enough to justify FFI packaging.
- No dependency on `taru-api`, `taru-server`, `taru-core`,
  `taru-streaming`, or `taru-transcode`.

### ACF-030: Minimal Media Library Browse Loop

Status: complete
Owner: codex
Depends on: ACF-020

Expose the smallest browse flow needed to choose playable content.

Scope:

- Media Library list.
- Media Item list.
- Media Item detail.
- Main shell destinations for Home, Libraries, Search, and Settings.
- Shared phone/tablet route model with responsive grids and detail context
  instead of separate tablet-only product behavior.
- Home as a playback launchpad with Libraries and Search as stable anchors.
- Continue Watching, Latest, and Next Up only when backed by Public Client API
  data.
- Search entry with global title-oriented query, basic result grouping,
  empty/error states, and navigation to detail pages.
- Browse Facet Result for supported genre, tag, person, studio, collection,
  year, and item-kind facets exposed by the Public Client API.
- Detail metadata relationships, including genre, tag, cast/crew preview,
  studio, collection, and hierarchy entries, should navigate to detail or
  Browse Facet Result routes instead of one-off pages.
- Actionable browse/search/detail empty states for empty library, no search
  results, loading timeout, permission denied, no playable source, and source
  unavailable.
- No advanced multi-condition filter builder, arbitrary database-column
  browsing, editable people/collection pages, saved searches, search history,
  provider-specific search, or cross-server search in the first slice.
- Settings for server identity, re-authentication, connection diagnostics,
  theme, basic playback/subtitle preference, mobile-network behavior, About,
  license, and version information.
- Detail routes for Library, Media Item, Series, Season, and Episode as
  supported by current Public Client API data.
- Media Item Detail as a playback decision surface with Play/Resume, basic
  metadata, hierarchy navigation, and explainable no-source/error states.
- Search or basic filtering only if needed for the first playback loop.
- Placeholder artwork behavior until Managed Artwork routes are complete.

Validation:

- Mocked API tests for pagination and empty/error states.
- Manual debug build can navigate from server connection to item detail.

#### ACF-030A: Minimal Library Browse Tracer

Status: complete
Owner: codex
Completed: 2026-05-17

This sub-slice proves active-server-scoped browse without claiming the full
ACF-030 browse-to-detail loop.

Scope completed:

- Direct Kotlin Public Client API browse client for `GET /libraries` and the
  smallest `GET /items?limit=&offset=` tracer.
- Kotlin DTO mirrors for the minimal `LibraryListResponse`, `ItemsResponse`,
  `PageInfo`, `LibraryDto`, and `MediaItemDto` fields needed by the first
  browse shell.
- Home/Libraries Compose shell after a saved active server profile exists.
- Loading, empty, unauthorized, unreachable, forbidden, unsupported-version,
  public API error, invalid-response, and missing-token states.
- Mocked API unit tests for pagination, empty library state input,
  unauthorized diagnostics, unreachable diagnostics, public error redaction,
  active-server base URL switching, and token-reference isolation.

Explicit non-goals preserved:

- No Media3 playback.
- No playback decision or playback request construction.
- No UniFFI implementation.
- No downloads/offline.
- No external-player handoff.
- No advanced search/facets.
- No dependency on server/internal Rust crates from Android.

Remaining ACF-030 work:

- Library detail route.
- Search shell and result navigation.
- Browse Facet Result route for public API backed detail and library facets.
- Settings shell beyond server switching through the setup surface.
- Manual debug walkthrough from server connection to item detail against a
  running Taru server fixture.

#### ACF-030B: Minimal Media Item Detail Tracer

Status: complete
Owner: codex
Completed: 2026-05-17

This sub-slice proves active-server-scoped Media Item detail loading without
starting playback work.

Scope completed:

- Direct Kotlin Public Client API browse client method for
  `GET /items/{item_id}`.
- Minimal Android DTO mirrors for `ItemDetailResponse`, including Media Item,
  Media Sources, credits, genres, tags, collections, studios, and image count
  fields needed by the read-only detail surface.
- Home/Libraries item rows and posters navigate to a read-only Media Item
  detail screen.
- Detail screen shows client-safe Canonical Metadata, metadata chips, related
  response counts, loading, missing item, missing-token, unauthorized,
  forbidden, unreachable, unsupported-version, public error, and
  invalid-response states.
- Mocked API unit tests for successful detail decoding, forbidden detail
  diagnostics, unreachable diagnostics through browse coverage,
  unsupported-version rejection, invalid response, missing item, public error
  redaction, active-server base URL switching, and token-reference isolation.

Explicit non-goals preserved:

- No Media3 playback.
- No Play/Resume activation.
- No playback decision or playback request construction.
- No Source / Version Picker behavior.
- No UniFFI implementation.
- No downloads/offline.
- No external-player handoff.
- No search/facets.
- No dependency on server/internal Rust crates from Android.

Remaining ACF-030 work:

- Compose UI baseline rewrite before adding more browse surfaces.
- Library detail route, if needed before playback.
- Search shell and result navigation.
- Browse Facet Result route for public API backed genre, tag, person, studio,
  collection, year, and item-kind facets.
- Settings shell beyond server switching through the setup surface.
- Manual debug walkthrough from server connection to item detail against a
  running Taru server fixture.

#### ACF-030C: Compose UI Baseline Rewrite

Status: done_with_concerns
Owner: codex
Depends on: ACF-030B
Implemented: 2026-05-18

Replace the tracer-oriented Compose browse/detail shell with a production
baseline aligned with `CLIENT_INTERFACE_DESIGN.md` and the reference
screenshots.

Rationale:

- `ACF-030A` and `ACF-030B` intentionally proved API, active-server, token, and
  diagnostics behavior before the final UI direction existed.
- The connection, profile, token, Public Client API client, DTO, diagnostics,
  and unit-test foundations are valuable and should be retained.
- The current `TaruBrowseShell` and read-only detail surface are tracer UI.
  Continuing to add Search, Facet, and Settings on top of that structure would
  create avoidable UI debt.
- The implementation may replace tracer UI files outright. Compatibility with
  temporary tracer screen structure is not required; preserving public client
  behavior, token safety, and test coverage is required.

Scope:

- Keep existing connection/auth clients, token vault, server profile storage,
  browse client, DTOs, and tests.
- Introduce a real app shell with top-level destinations: Home, Libraries,
  Search, and Settings.
- Replace tab-based Home/Libraries with Material 3 bottom navigation on phone
  and a route model that can later support tablet navigation rail/detail panes.
- Extract reusable Compose components for:
  - app scaffold and destination navigation;
  - section headers;
  - poster cards and compact media rows;
  - library cards;
  - metadata and facet chips;
  - source summary placeholders;
  - empty, loading, and error states;
  - settings groups and server profile cards.
- Rebuild Home and Libraries with the initial reference screenshots as the
  visual baseline while preserving current API-backed content.
- Rebuild Media Item Detail as a playback decision layout skeleton using
  current read-only detail data. Play/Resume and Source / Version Picker remain
  visual placeholders only until `ACF-040`.
- Add Settings Home and Server Profile screens using existing profile/token
  state where possible, without exposing token values.
- Add first-version-safe expressive behavior: selected-state transitions,
  subtle press feedback, sheet transitions, inline loading/error feedback, and
  local artwork-derived muted accents where contrast and fallback behavior are
  explicit.
- Keep Search and Browse Facet Result as navigable placeholder destinations
  only if they are needed to prove shell structure; public search/facet API
  integration remains a follow-on sub-slice.

Explicit non-goals:

- No Media3 playback.
- No playback decision or request construction.
- No Source / Version Picker behavior beyond visual placeholder entry points.
- No search API integration unless already available and trivial.
- No advanced facets, editable people/collection pages, or admin surfaces.
- No complex choreography, global dynamic theme replacement, or required alpha
  Material Expressive API dependency.
- No new dependency on server/internal Rust crates.

Validation:

- Existing Android unit tests for connection, browse, and detail remain green.
- Android debug build passes.
- Manual debug walkthrough can still connect to a server, load browse content,
  open a Media Item detail, switch server through Settings or server profile
  flow, and return to browse.
- UI never displays access-token values, raw diagnostics, local filesystem
  paths, FFmpeg commands, provider payloads, or server-local paths.

Implementation completed:

- Replaced the tracer tab shell with a Material 3 `Scaffold`, bottom
  navigation, explicit route model, animated route transitions, and
  top-level Home, Libraries, Search, and Settings destinations.
- Split the former monolithic browse shell into route/state, reusable
  components, Home, Libraries, Media Item Detail, Settings, placeholders,
  formatters, and preview files.
- Rebuilt Home and Libraries using API-backed library and media-item data,
  retained current loading/empty/error behavior, and added subtle press and
  loading feedback.
- Rebuilt Media Item Detail as a playback-decision skeleton with disabled
  Play/Source placeholders, source summary, metadata chips, Cast & Crew, and
  relationship rows.
- Added Settings Home and Server Profile screens backed by the existing server
  profile snapshot and token vault without displaying token values.
- Added Compose Material Icons Extended for standard Material iconography.

Validation completed:

- `apps/android/gradlew.bat -p apps/android :app:testDebugUnitTest` passed on
  2026-05-18.
- `apps/android/gradlew.bat -p apps/android :app:assembleDebug` passed on
  2026-05-18.
- `git diff --check` passed on 2026-05-18 with Windows line-ending
  normalization warnings only.

Concern:

- Manual debug walkthrough against a running Taru server fixture and Android
  device/emulator was not executed in this session. ACF-030 remains open until
  that walkthrough and the remaining search/facet route behavior are validated.

#### ACF-030D: Search, Facet, And Walkthrough Hardening

Status: complete
Owner: codex
Depends on: ACF-030C
Completed: 2026-05-18

Finish the remaining non-playback browse loop before moving into playback
decision construction.

Scope:

- Check current Public Client API support for search, library-scoped item
  browsing, and supported Browse Facets.
- Wire Search and Browse Facet Result to public API routes when the contract is
  already explicit; otherwise record the API gap as a follow-up before adding
  client-only semantics.
- Add or defer Library Detail based on public API support for library-scoped
  item pages and facets.
- Run and record a manual debug walkthrough from server connection to browse,
  item detail, Settings, Server Profile, and back to browse.
- Keep the UI baseline from ACF-030C intact; this task should harden routes and
  behavior, not start a new visual redesign.

Explicit non-goals:

- No Media3 playback.
- No playback decision or request construction.
- No advanced query builder, saved searches, search history, provider-specific
  search, metadata editing, or server administration.

Validation:

- Mocked Android tests cover any newly wired search/facet routes.
- Manual debug app walkthrough evidence is recorded in
  `EVIDENCE_AND_GATES.md`.
- `apps/android/gradlew.bat -p apps/android :app:testDebugUnitTest` passes.
- `apps/android/gradlew.bat -p apps/android :app:assembleDebug` passes.

Implementation completed on 2026-05-18:

- Added direct Kotlin Public Client API calls and Android DTO mirrors for:
  - `GET /search?q=&facet=&limit=&offset=`;
  - `GET /genres/{genre_id}/items`;
  - `GET /tags/{tag_id}/items`;
  - `GET /people/{person_id}/items`.
- Added mocked Android unit tests for search query/facet/pagination encoding,
  search hit decoding, genre item result decoding, tag/person item route
  construction, active profile use, and safe request redaction.
- Replaced the Search top-level placeholder with a real submitted-query Search
  screen backed by `/search`.
- Replaced the Browse Facet placeholder with a route that loads API-backed
  Genre, Tag, and Person targets only when a stable facet id is available.
- Detail metadata chips now map Genre/Tag display labels to stable relation ids
  from `ItemDetailResponse.genres` and `ItemDetailResponse.tags` by response
  order when available.
- Detail person relationships use `credits.person_id` only when present; they
  do not invent cast or crew names from missing data.
- Unsupported browse targets are explicit API-gap pages rather than
  client-only pseudo filters.

API gaps recorded:

- Library-scoped item browsing and library facet result routes.
- Studio facet item route.
- Collection facet item route.
- Year facet route.
- Media Item kind facet route.
- Rich person/credit detail data with display names and role-specific browsing.

Validation completed on 2026-05-18:

- `cargo fmt --all -- --check` passed.
- `cargo nextest run -p taru-server http::tests::catalog --no-fail-fast`
  passed.
- `cargo build -p taru-server` passed with pre-existing unused-code warnings.
- `apps/android/gradlew.bat -p apps/android :app:testDebugUnitTest` passed.
- `apps/android/gradlew.bat -p apps/android :app:assembleDebug` passed.
- `git diff --check` passed with Windows line-ending normalization warnings
  only.
- Manual debug walkthrough passed on
  `Pixel_3a_API_34_extension_level_7_x86_64` against a real local
  `taru-server` fixture on `127.0.0.1:3018` with `adb reverse tcp:3018`.
  The flow covered connection/setup, Home, Libraries, Media Item Detail,
  Search, Genre Browse Facet Result, Settings, Server Profile, and return to
  Home.
- The walkthrough exposed and verified a server-side `/search` pagination
  query parsing fix. `SearchPageQuery` now owns explicit `limit` and `offset`
  fields instead of relying on flattened pagination under the `/search` query
  shape.

### ACF-040: Playback Decision And Request Construction

Status: complete
Owner: codex
Depends on: ACF-030

Connect item/source selection to Taru playback decision APIs and construct
public playback requests.

Scope:

- Source selection from an item detail.
- Source / Version Picker when multiple playable sources or variants are
  available.
- Picker fields for client-safe label, Media Library/source context,
  container, video codec, audio codec, resolution, HDR, bitrate, track counts,
  playback-mode preview, and warnings when available.
- Explicit exclusion of local paths, secret references, raw provider payloads,
  FFmpeg commands, and server-local output paths.
- Playback capability query model.
- Direct, remux, and HLS request construction.
- User-facing mapping for common playback decision failures.

Validation:

- Tests cover URL/path/query/header construction.
- Tests prove bearer token values are never logged or shown in diagnostic
  output.

Implementation completed on 2026-05-18:

- Added a direct Kotlin Public Client API playback client for
  `GET /sources/{source_id}/playback/decision`.
- Added Android DTO mirrors for `PlaybackDecisionResponse`,
  `ClientPlaybackDecision`, direct-play plans, transcode plans, media probe
  facts, stream facts, and the small enum set needed by ACF-040.
- Added playback request target builders for direct stream, direct HEAD
  preflight, remux stream, HLS playlist, and HLS segment routes.
- Added recommended target selection from playback decision mode:
  direct play -> `/stream`, remux -> `/stream/remux`, transcode -> HLS
  playlist.
- Wired Media Item Detail source selection to request playback decisions and
  display client-safe route previews without starting Media3 playback.
- Added a first Source / Version Picker surface by listing candidate Media
  Sources and letting the user request a decision for any candidate.
- Kept access-token values, source locators, server-local paths, FFmpeg
  commands, and raw provider payloads out of diagnostics and safe request
  previews.

Validation completed on 2026-05-18:

- `apps/android/gradlew.bat -p apps/android :app:testDebugUnitTest --tests dev.taru.android.playback.TaruPlaybackClientTest`
  passed.
- `apps/android/gradlew.bat -p apps/android :app:testDebugUnitTest` passed.
- `apps/android/gradlew.bat -p apps/android :app:assembleDebug` passed.
- `git diff --check` passed with Windows line-ending normalization warnings
  only.
- Manual real-server/device walkthrough passed on
  `Pixel_3a_API_34_extension_level_7_x86_64` against a real local
  `taru-server` fixture on `127.0.0.1:3018` with
  `adb reverse tcp:3018 tcp:3018`. The flow covered setup save, Home,
  `Night Harbor` Media Item Detail, `Playback Source Selection`, source
  candidate `Night Harbor.mkv`, `Request decision`, `HLS route prepared`, a
  safe HLS playlist route preview, and the explicit handoff copy
  `Playback starts in ACF-050.`
- The walkthrough UI dump contained expected playback decision text and did
  not contain the raw walkthrough token, `file:///`, `G:/`, or `ffmpeg`.
- Direct HEAD preflight remains covered by request-construction tests; visible
  preflight/player behavior is deferred to `ACF-050`.

### ACF-050: Media3 Playback Smoke Slice

Status: complete
Owner: codex
Depends on: ACF-040

Play one Taru public playback route through Media3 ExoPlayer.

Scope:

- Media3 player screen.
- Track / Subtitle Sheet.
- Playback Error Sheet.
- Play/pause, seek bar, loading/buffering, elapsed/remaining time,
  full-screen/orientation handling, and clear exit behavior.
- Playback session cancellation on exit when the selected route creates a
  cancellable session.
- Actionable playback errors for unsupported media, server playback processing
  failure, expired/cancelled session, and network interruption.
- Player lifecycle tied to Android lifecycle.
- First route target selected from the ACF-040 playback request result.
- Minimal loading, error, play/pause, seek, and full-screen behavior.

Validation:

- Manual playback smoke test against a local Taru server fixture.
- Instrumented or integration test plan recorded if local CI cannot run
  emulator playback.

Implementation completed on 2026-05-18:

- Added AndroidX Media3 ExoPlayer, HLS, and UI dependencies.
- Added a minimal full-screen Media3 player route launched from the ACF-040
  prepared playback target.
- The player builds a Media3 HTTP data source from the real playback request,
  including authorization headers, while UI/debug output uses only the safe
  redacted request preview.
- Added a `PlayerView`-backed Compose surface with back navigation, Media3
  controller controls, elapsed/duration UI, buffering/playing/paused/ended
  state copy, and error code copy.
- Player lifecycle is tied to route disposal; ExoPlayer is released when the
  user exits the route.
- Kept resume/progress reporting, track/subtitle sheet depth, PiP, cast,
  downloads, and external-player handoff out of this smoke slice.

Validation completed on 2026-05-18:

- `apps/android/gradlew.bat -p apps/android :app:compileDebugKotlin --no-daemon --rerun-tasks`
  passed.
- `apps/android/gradlew.bat -p apps/android :app:testDebugUnitTest --tests dev.taru.android.player.PlaybackLaunchTest --no-daemon`
  passed.
- `apps/android/gradlew.bat -p apps/android :app:testDebugUnitTest --no-daemon`
  passed.
- `apps/android/gradlew.bat -p apps/android :app:assembleDebug --no-daemon`
  passed.
- Manual real-server/device playback smoke passed on
  `Pixel_3a_API_34_extension_level_7_x86_64` against a local
  `taru-server` fixture on `127.0.0.1:3018` with
  `adb reverse tcp:3018 tcp:3018`.
- Smoke media: local `Night Harbor.mkv`, regenerated as a 2 second H.264/AAC
  Matroska file and re-probed by `scan-all`.
- Playback route selected by server: remux target
  `/sources/{source_id}/stream/remux?output_container=mp4`.
- Observed UI state: Media3 player route opened, PlayerView controller showed
  `00:02 / 00:02`, and app state copy showed `Media3: Ended`.
- Observed logcat state: ExoPlayer initialized as AndroidX Media3 `1.10.1`;
  H.264 video decoder and AAC audio decoder were created; no playback error
  was observed in the filtered logcat output.
- Raw access token was not shown in the player UI.

### ACF-060: Playback Session And Resume Follow-Up

Status: complete
Owner: codex
Depends on: ACF-050
Completed: 2026-05-18

Define the next server/client contract needed for resume playback and playback
state.

Scope:

- Audit existing Public Client API coverage for session inspection and
  cancellation.
- Identify missing progress/resume routes, if any.
- Define periodic position reporting and final-position-on-exit behavior when
  Public Client API support exists.
- Define active-server-scoped device-local transient position behavior when
  Public Client API support is incomplete.
- Ensure local transient position never appears as cross-device Continue
  Watching.
- Keep **User Playback State** terminology aligned with `CONTEXT.md`.

Validation:

- Follow-up API workstream or ADR is created only if a hard-to-change contract
  is needed.
- Tests or design notes prove playback state does not mix across server
  profiles.

Implementation completed on 2026-05-18:

- Audited Public Client API coverage and confirmed the current public surface
  includes playback session inspection and cancellation:
  - `GET /playback/sessions/{session_id}`;
  - `POST /playback/sessions/{session_id}/cancel`.
- Confirmed the current Public Client API does not expose authoritative
  progress reporting, resume lookup, or **User Playback State** routes.
- Added Android DTO mirrors for `TranscodeSessionResponse`,
  `TranscodeSessionDto`, transcode session kind/state, and failure category.
- Added Android playback client methods for session inspection and
  cancellation using the public routes above, with safe request previews and
  token redaction.
- Added `PlaybackLaunchRequest` metadata for active server profile, Media
  Item, Media Source, playback mode, optional session id, and device-local
  resume position.
- Added a device-local transient playback position store keyed by
  active-server profile id, Media Item id, and Media Source id.
- Wired the Media3 player route to seek to device-local transient position on
  launch, save final transient position on exit, clear it on ended/zero
  position, and request playback session cancellation only when a session id
  is explicitly known.

API gaps recorded:

- No public progress-reporting route exists yet.
- No public resume lookup route exists yet.
- No public authoritative **User Playback State** route exists yet.
- Current remux/HLS playback stream responses do not expose a session id to
  Android. HLS segment URLs contain session ids after playlist generation, but
  the Android launch path should not parse playlists to invent lifecycle
  handles. Exit cancellation is therefore implemented only for launch requests
  that already carry an explicit session id.

Validation completed on 2026-05-18:

- `apps/android/gradlew.bat -p apps/android :app:testDebugUnitTest --tests dev.taru.android.player.PlaybackLaunchTest --no-daemon`
  passed.
- `apps/android/gradlew.bat -p apps/android :app:testDebugUnitTest --tests dev.taru.android.playback.TaruPlaybackClientTest --no-daemon`
  passed.
- `apps/android/gradlew.bat -p apps/android :app:testDebugUnitTest --no-daemon`
  passed.
- `apps/android/gradlew.bat -p apps/android :app:assembleDebug --no-daemon`
  passed.
- `git diff --check` passed with Windows line-ending normalization warnings
  only.

## Open Follow-Ups

- Offline download manager and background sync.
- Offline/downloads lifecycle design covering source selection, original vs
  remux/HLS/Optimized Version output, remote-source behavior, user/server
  binding, subtitles/audio/artwork storage, disk budget, deletion, resume,
  corruption recovery, and User Playback State reconciliation.
- Cast/route integration.
- External player handoff using a short-lived playback token or secure handoff
  URL; do not expose long-lived bearer tokens to external Android apps.
- Android TV focus and ten-foot UI.
- iOS shell implementation.
- Shared Rust mobile-core crate and UniFFI packaging, if not introduced during
  ACF-020.
