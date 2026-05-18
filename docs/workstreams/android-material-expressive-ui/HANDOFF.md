# Android Material Expressive UI Handoff

Status: Active
Last updated: 2026-05-18

## Current State

The workstream is open. `AME-010` froze the scope and `AME-020` has been
completed and validated.

The Android Client Foundation is complete and already provides:

- connection/auth flow with secure token references;
- active-server scoped browse/search/detail;
- playback decision request construction;
- Media3 playback smoke;
- playback session inspect/cancel client methods;
- device-local transient playback position boundaries.

## Next Task

Run `AME-030`: implement the V2 Home and browse surfaces on top of the new
foundation.

Recommended implementation order:

1. Keep the new theme and shell as the only app chrome foundation.
2. Rework Home, Libraries, and Browse Facet Result around artwork-led, dense,
   media-first composition.
3. Preserve API-backed facets only, and keep fake Continue Watching out.
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

## Open Risks

- Dynamic color may reduce Taru brand recognizability if applied globally.
  Mitigation: keep it optional and use static Taru dark roles as default.
- Artwork-derived accents can hurt contrast. Mitigation: provide contrast-safe
  fallbacks and keep accents local.
- Screen rewrite can accidentally change Public Client API assumptions.
  Mitigation: preserve existing client tests and add UI-facing tests only where
  behavior changes.
