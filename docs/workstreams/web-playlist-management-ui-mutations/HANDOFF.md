# Web Playlist Management UI Mutations - Handoff

Status: Active
Last updated: 2026-05-29

## Current State

This lane is open. The backend/Public Client User Playlist contract is already
closed in `docs/workstreams/user-playlists-contract-and-web-slice/`. The first
web slice can list playlists and items at `/media/my-list` through Public
Client live data with fixture fallback. WPMU-020 added the web mutation
boundary: Public Client-backed data-source methods and TanStack Query mutation
hooks now cover playlist create, rename, delete, add item, remove item, and
reorder. WPMU-030 added `/media/my-list` create, rename, and delete controls
on top of those hooks.

The remaining lane work is item membership and ordering UI. It must keep
playlist management on the Public Client boundary and must not import Admin API
code into media features.

## Active Task

- Task ID: WPMU-040
- Owner: Codex
- Files: `web/src/features/media`, `web/src/test`
- Validation: `npm --prefix web run test`; `npm --prefix web run check`
- Status: READY
- Review: inaccessible item facts must not leak, fixture fallback must be
  truthful, and no media source/library-file writes may be introduced.
- Evidence: route/data-source tests and browser smoke

## Decisions Since Last Update

- This lane does not redesign the Public Client route contract.
- Fixture mode may preview forms/states, but cannot claim persisted mutation
  success.
- Fixture mutation payloads explicitly return `persisted: false`.
- Playlist mutation hooks invalidate the playlist list and affected item list;
  delete also removes the deleted playlist item query cache.
- CRUD controls are owned by `my-list-page.tsx`; shell routing remains on the
  existing `onRouteStateChange` contract.
- Deleting the active playlist moves route state to the next available
  playlist, or clears `playlist` when none remains.
- Reorder starts with explicit accessible controls; drag-and-drop is optional
  and should be split if it expands cost.

## Blockers

- None known.

## Next Recommended Action

Start WPMU-040 with TDD: add item removal from playlist rows/cards and a narrow
add-to-playlist entry point from browse/detail, using the existing Public
Client mutation hooks and preserving fixture non-persistence feedback.
