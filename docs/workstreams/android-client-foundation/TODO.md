# Android Client Foundation TODO

Status: Proposed
Last updated: 2026-05-17

## Task Ledger

### ACF-010: Android Scaffold And Repository Boundary

Status: pending
Owner: unassigned

Create the initial Android project under `apps/android` with a minimal Kotlin
and Compose app shell. Keep it outside the Rust Cargo workspace.

Scope:

- Android Gradle project scaffold.
- App entry point and one local debug screen.
- Basic dark-first Compose Material 3 theme aligned with Design Language v0 in
  `UX_CONTEXT.md`, including initial color roles, spacing, type roles, poster
  aspect ratios, corner radius, touch targets, and component semantics.
- README with local Android prerequisites and build command.
- No Taru server calls yet.

Validation:

- Android debug build command is documented and passes on a configured Android
  toolchain.
- Existing Rust workspace checks still ignore Android app files unless
  explicitly requested.

### ACF-020: Public Client Connection And Auth Slice

Status: pending
Owner: unassigned
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

- Mocked HTTP tests for success, unreachable server, unauthorized, and version
  mismatch.
- Tests prove switching active server changes request base URL and does not
  mix cached state between profiles.
- Tests prove token values are hidden and never included in diagnostics or
  safe request previews.
- Tests cover sanitized diagnostics for representative public error envelopes.
- No dependency on `taru-api`, `taru-server`, `taru-core`,
  `taru-streaming`, or `taru-transcode`.

### ACF-030: Minimal Media Library Browse Loop

Status: pending
Owner: unassigned
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
- Actionable browse/search/detail empty states for empty library, no search
  results, loading timeout, permission denied, no playable source, and source
  unavailable.
- No advanced facets, sort controls, saved searches, search history,
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

### ACF-040: Playback Decision And Request Construction

Status: pending
Owner: unassigned
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

### ACF-050: Media3 Playback Smoke Slice

Status: pending
Owner: unassigned
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

### ACF-060: Playback Session And Resume Follow-Up

Status: pending
Owner: unassigned
Depends on: ACF-050

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
