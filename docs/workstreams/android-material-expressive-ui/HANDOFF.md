# Android Material Expressive UI Handoff

Status: Active
Last updated: 2026-05-18

## Current State

The workstream is open. `AME-010`, `AME-020`, `AME-030`, `AME-040`, and
`AME-050` have been completed and validated.

The Android Client Foundation is complete and already provides:

- connection/auth flow with secure token references;
- active-server scoped browse/search/detail;
- playback decision request construction;
- Media3 playback smoke;
- playback session inspect/cancel client methods;
- device-local transient playback position boundaries.

## Next Task

Run `AME-060`: verify the completed V2 UI rewrite, update final evidence, and
close or split follow-on work.

Recommended implementation order:

1. Run fresh Android unit tests, debug assemble, and diff hygiene checks.
2. Run broader Rust gates if closeout touches shared/public API files.
3. Review remaining API gaps and split follow-ons instead of hiding them in
   Android-only behavior.
4. Close the workstream or record explicit deferred V3 exploration.

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
- `ui/browse/TaruBrowseShell` now routes through the new shell.
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
- `TaruBrowseShell` still owns Public Client API playback decision requests and
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

- Dynamic color may reduce Taru brand recognizability if applied globally.
  Mitigation: keep it optional and use static Taru dark roles as default.
- Artwork-derived accents can hurt contrast. Mitigation: provide contrast-safe
  fallbacks and keep accents local.
- Screen rewrite can accidentally change Public Client API assumptions.
  Mitigation: preserve existing client tests and add UI-facing tests only where
  behavior changes.
