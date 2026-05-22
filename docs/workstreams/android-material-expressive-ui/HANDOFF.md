# Android Material Expressive UI Handoff

Status: Completed
Last updated: 2026-05-18

## Current State

The workstream is closed. `AME-010`, `AME-020`, `AME-030`, `AME-040`,
`AME-050`, and `AME-060` have been completed and validated.

The Android Client Foundation is complete and already provides:

- connection/auth flow with secure token references;
- active-server scoped browse/search/detail;
- playback decision request construction;
- Media3 playback smoke;
- playback session inspect/cancel client methods;
- device-local transient playback position boundaries.

## Closeout Result

`AME-060` verified the completed V2 UI rewrite and closed the lane.

Fresh closeout gates passed on 2026-05-18:

- `apps\android\gradlew.bat -p apps\android :app:testDebugUnitTest --no-daemon`
- `apps\android\gradlew.bat -p apps\android :app:assembleDebug --no-daemon`
- `cargo fmt --all -- --check`
- `cargo nextest run --workspace --no-fail-fast` with 364 tests passed
- `git diff --check`

## Constraints To Preserve

- Keep Public Client API clients and DTO boundaries intact.
- Keep token values out of UI, logs, diagnostics, and tests.
- Do not invent authoritative User Playback State.
- Do not fake unsupported facets or local filtering as server-backed results.
- Do not introduce V3 irregular layout complexity in this lane.
- Do not depend on AGPL server/internal crates from Android.

## AME-020 Outcome

- `ui/theme` now supports optional dynamic color and artwork-accent hooks.
- Shared `ui/components` hold the reusable surface, state, badge, and motion
  vocabulary.
- `ui/shell` now provides the adaptive phone bottom navigation and tablet rail.
- `ui/browse/NakoBrowseShell` now routes through the new shell.
- JVM tests cover artwork accent determinism.

## AME-030 Outcome

- `HomeScreen` now acts as a playback-first launchpad with real server/library
  context, stable library/search anchors, and visible Media Items.
- Home no longer shows fake Continue Watching or unsupported facet shortcuts.
- `LibrariesScreen` now keeps structural Media Libraries first and shows
  visible Media Items as a media-led grid.
- `BrowseFacetRouteContent` now has an artwork-led facet header and only treats
  API-backed Genre, Tag, and Person relationship results as real result pages.
- Unsupported facet families remain explicit API-gap states.

## AME-040 Outcome

- `MediaItemDetailScreen` was replaced by `ui/screens/detail/MediaItemDetailRoute`.
- Detail now presents an artwork-led playback decision hero, Play/Resume action
  hierarchy, explicit device-local resume wording, overview, metadata chips,
  Cast & Crew preview, and Related Media rows.
- Source / Version selection now lives in
  `ui/screens/sourcepicker/SourcePickerScreen` as a picker-style surface with
  selected-source state, client-safe source facts, and playback-mode
  consequences for Direct, Remux, HLS, and Transcode.
- Source picker tests prove visible source facts do not include server-local
  locators.
- `NakoBrowseShell` still owns Public Client API playback decision requests and
  Media3 playback launch construction.

## AME-050 Outcome

- `PlaybackPlayerScreen` was replaced by
  `ui/screens/player/PlaybackPlayerRoute`.
- Player now presents an immersive overlay with title, playback mode, loading
  status, selected source context, local resume wording, and session state.
- Playback errors now use a sheet-style recovery surface with Retry, Back to
  detail, and copy-safe diagnostics built from `SafeRequestPreview`.
- Media3 PlayerView setup, device-local position persistence, and playback
  session cancellation behavior remain in place.
- Settings and Server Profile now live in `ui/screens/settings`, with grouped
  surfaces for active server, account access, playback, tracks/subtitles,
  diagnostics, profiles, and sign-out.
- JVM presentation tests cover player diagnostics redaction, local resume
  wording, and settings diagnostics safety.

## Open Risks

- Dynamic color may reduce Nako brand recognizability if applied globally.
  Mitigation: keep it optional and use static Nako dark roles as default.
- Artwork-derived accents can hurt contrast. Mitigation: provide contrast-safe
  fallbacks and keep accents local.

## Follow-Ons

- V3 irregular/freeform geometry remains deferred until the V2 baseline has
  shipped and can be evaluated against real use.
- Authoritative User Playback State and cross-device Continue Watching need a
  Public Client API/server workstream before Android can present them as real
  state.
- Richer source technical facts, track/subtitle selection, chapters, and
  source-level diagnostics need explicit Public Client API support before the
  Android UI can promote them to first-class controls.
- Downloads/offline playback, external player handoff, picture-in-picture, and
  advanced player gestures remain separate Android/player workstreams.
- Deprecated `LocalClipboardManager` usage should be migrated when the Compose
  clipboard replacement is clear for this app baseline.
