# Web Route-Owned Product Surfaces

Status: Active
Last updated: 2026-05-28

## Why This Lane Exists

Top-level routes are TanStack-owned, but Media/Admin still contain many
internal page states. That hides product URLs, weakens browser navigation, and
makes live API wiring harder to reason about.

## Target State

- Media search, item detail, library, and player-adjacent surfaces have explicit
  route ownership.
- Admin libraries, users, tasks, logs, settings, and addon-manager entry points
  have explicit route ownership.
- Route params/search params own durable navigation state.
- Feature components become route leaves or route-local panels instead of a
  single switch-heavy surface.

## In Scope

- Add TanStack child routes for accepted product surfaces.
- Preserve fixture/live data seams.
- Add route contract tests for each new route.
- Keep deferred domains out of route ownership until accepted.

## Out Of Scope

- Implementing new backend behavior.
- Native playback.
- Full visual redesign.

## Closeout Condition

This lane can close when the accepted Media/Admin child surfaces are deep
linkable, tests pass, and internal view-state routing is reduced to local UI
state only.
