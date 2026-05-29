# User Playlists Contract And Web Slice

Status: Active
Last updated: 2026-05-28

This lane defines the backend/Public Client contract that must exist before the
new `web/` frontend can restore any Playlist or My List UI. It is opened from
`WDRP-050` because user principal, User Playback State, and Library Access
prerequisites are now present, but no user playlist public contract exists yet.

## Authoritative Docs

- `DESIGN.md` - problem, scope, non-goals, and architecture direction.
- `CONTRACT_READINESS.md` - readiness decision and contract prerequisites.
- `TODO.md` - executable task ledger.
- `EVIDENCE_AND_GATES.md` - validation commands and evidence log.
- `HANDOFF.md` - current state and next action.

## Current Execution Point

`UPCW-010` opened this lane from `WDRP-050`. Continue with `UPCW-020`, the
contract freeze for User Playlist routes and DTOs before any `web/` UI work.
