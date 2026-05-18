# Android Material Expressive UI Handoff

Status: Active
Last updated: 2026-05-18

## Current State

The workstream is open. `AME-010`, `AME-020`, and `AME-030` have been completed
and validated.

The Android Client Foundation is complete and already provides:

- connection/auth flow with secure token references;
- active-server scoped browse/search/detail;
- playback decision request construction;
- Media3 playback smoke;
- playback session inspect/cancel client methods;
- device-local transient playback position boundaries.

## Next Task

Run `AME-040`: implement the V2 Media Item Detail and Source / Version Picker
on top of the new foundation.

Recommended implementation order:

1. Keep the new theme and shell as the only app chrome foundation.
2. Rework Media Item Detail around clear playback decision hierarchy.
3. Move source/version choice toward a picker-style surface without leaking
   server-local paths.
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

## Open Risks

- Dynamic color may reduce Taru brand recognizability if applied globally.
  Mitigation: keep it optional and use static Taru dark roles as default.
- Artwork-derived accents can hurt contrast. Mitigation: provide contrast-safe
  fallbacks and keep accents local.
- Screen rewrite can accidentally change Public Client API assumptions.
  Mitigation: preserve existing client tests and add UI-facing tests only where
  behavior changes.
