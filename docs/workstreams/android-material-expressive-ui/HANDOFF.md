# Android Material Expressive UI Handoff

Status: Active
Last updated: 2026-05-18

## Current State

The workstream is open. `AME-010`, `AME-020`, `AME-030`, and `AME-040` have
been completed and validated.

The Android Client Foundation is complete and already provides:

- connection/auth flow with secure token references;
- active-server scoped browse/search/detail;
- playback decision request construction;
- Media3 playback smoke;
- playback session inspect/cancel client methods;
- device-local transient playback position boundaries.

## Next Task

Run `AME-050`: implement the V2 Player and Settings surfaces on top of the new
foundation.

Recommended implementation order:

1. Keep the new theme and shell as the only app chrome foundation.
2. Rework Player chrome and playback error handling without changing Media3
   launch/session boundaries.
3. Rework Settings Home and Server Profile with restrained diagnostics and
   token-safe copy.
4. Run Android unit tests, Android debug assemble, and `git diff --check`.

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

## Open Risks

- Dynamic color may reduce Taru brand recognizability if applied globally.
  Mitigation: keep it optional and use static Taru dark roles as default.
- Artwork-derived accents can hurt contrast. Mitigation: provide contrast-safe
  fallbacks and keep accents local.
- Screen rewrite can accidentally change Public Client API assumptions.
  Mitigation: preserve existing client tests and add UI-facing tests only where
  behavior changes.
