# Web Playlist Management UI Mutations - Milestones

Status: Active
Last updated: 2026-05-29

## M0 - Scope And Evidence Freeze

Exit criteria: scope, non-goals, assumptions, gates, and handoff are aligned.

Completed by `WPMU-010`.

## M1 - Public Client Mutation Boundary

Exit criteria: web data-source and TanStack Query mutation hooks cover the
existing playlist mutation SDK surface without Admin API imports or fixture
success overclaims.

Completed by `WPMU-020`.

## M2 - Playlist CRUD Controls

Exit criteria: `/media/my-list` supports create, rename, and delete with
route-safe state transitions and tested loading/error/empty states.

## M3 - Item Add And Remove Flows

Exit criteria: users can add and remove media items through Public Client-backed
flows without leaking inaccessible item facts or writing library/media source
state.

## M4 - Reorder And Conflict Handling

Exit criteria: playlist item order can be changed through an accessible flow
that submits full ordered membership and recovers from stale-version conflicts.

## M5 - Verification And Closeout

Exit criteria: shipped mutation behavior, validation evidence, residual risks,
and follow-ons are recorded.
