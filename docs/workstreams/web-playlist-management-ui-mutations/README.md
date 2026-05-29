# Web Playlist Management UI Mutations

Status: Closed
Last updated: 2026-05-29

This lane turns the closed User Playlist contract into real web playlist
management. `user-playlists-contract-and-web-slice` restored read-oriented
playlist browsing through Public Client live data; this lane adds the product
mutation flows without widening the playlist domain into sharing, smart lists,
or offline sync.

## Authoritative Docs

- `DESIGN.md` - problem, scope, non-goals, and architecture direction.
- `TODO.md` - executable task ledger.
- `MILESTONES.md` - milestone exit criteria.
- `EVIDENCE_AND_GATES.md` - validation commands and evidence log.
- `HANDOFF.md` - current state and next action.
- `CLOSEOUT.md` - shipped behavior, review result, residual risks, and follow-ons.

## Closeout

`WPMU-060` closed this lane on 2026-05-29. The web playlist management
surface now supports create, rename, delete, add item, remove item, and
explicit up/down reorder through the Public Client data-source and TanStack
Query mutation hooks.

Shared playlists, smart lists, drag-and-drop reorder, collaboration, offline
sync, and future Tauri/mobile playlist surfaces remain follow-ons rather than
blockers for this lane.
