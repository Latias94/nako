# Web Playlist Management UI Mutations

Status: Active
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

## Current Execution Point

`WPMU-010` opened the lane. Start `WPMU-020` by adding the web Public Client
mutation data-source and TanStack Query mutation boundary for create, rename,
delete, add, remove, and reorder.
