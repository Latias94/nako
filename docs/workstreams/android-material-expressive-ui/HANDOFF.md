# Android Material Expressive UI Handoff

Status: Active
Last updated: 2026-05-18

## Current State

The workstream is open. `AME-010` froze the scope and selected `AME-020` as the
first executable task.

The Android Client Foundation is complete and already provides:

- connection/auth flow with secure token references;
- active-server scoped browse/search/detail;
- playback decision request construction;
- Media3 playback smoke;
- playback session inspect/cancel client methods;
- device-local transient playback position boundaries.

## Next Task

Run `AME-020`: rewrite the UI foundation.

Recommended implementation order:

1. Introduce or refactor `ui/theme` tokens for static dark roles, optional
   dynamic color, motion, shape, spacing, type, and artwork accents.
2. Add `ui/components` for shared state cards, section headers, poster/backdrop
   surfaces, action clusters, and settings rows.
3. Add `ui/shell` for adaptive phone/tablet app chrome and route transitions.
4. Wire existing screens through the new shell with minimal screen redesign,
   preserving behavior.
5. Run Android unit tests, Android debug assemble, and `git diff --check`.

## Constraints To Preserve

- Keep Public Client API clients and DTO boundaries intact.
- Keep token values out of UI, logs, diagnostics, and tests.
- Do not invent authoritative User Playback State.
- Do not fake unsupported facets or local filtering as server-backed results.
- Do not introduce V3 irregular layout complexity in this lane.
- Do not depend on AGPL server/internal crates from Android.

## Open Risks

- Dynamic color may reduce Taru brand recognizability if applied globally.
  Mitigation: keep it optional and use static Taru dark roles as default.
- Artwork-derived accents can hurt contrast. Mitigation: provide contrast-safe
  fallbacks and keep accents local.
- Screen rewrite can accidentally change Public Client API assumptions.
  Mitigation: preserve existing client tests and add UI-facing tests only where
  behavior changes.
