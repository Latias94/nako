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
on top of those hooks. WPMU-040 added item removal from playlist list/card
views and a narrow add-to-playlist dropdown from media detail and browse cards.

The remaining lane work is item ordering UI. It must keep playlist management
on the Public Client boundary and must not import Admin API code into media
features.

## Active Task

- Task ID: WPMU-050
- Owner: Codex
- Files: `web/src/features/media`, `web/src/test`
- Validation: `npm --prefix web run test -- src/test/route-contracts.test.tsx src/test/route-state-contracts.test.tsx`; `npm --prefix web run check`
- Status: READY
- Review: reorder must submit full ordered `item_ids`, preserve route state,
  and refetch on conflicts.
- Evidence: state tests and browser smoke

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
- Removing a playlist item is available from list rows and poster cards and
  goes through `useRemoveUserPlaylistItemMutation`.
- The add-to-playlist control is shared between media detail and browse card
  entry points and uses `useAddUserPlaylistItemMutation`.
- Browse cards keep string media IDs instead of coercing IDs through
  `parseInt`, preserving nonnumeric Public Client IDs.
- Reorder starts with explicit accessible controls; drag-and-drop is optional
  and should be split if it expands cost.

## Blockers

- None known.

## Next Recommended Action

Start WPMU-050 with TDD: add explicit reorder controls for playlist items,
submit the full ordered `item_ids` payload, preserve the current route state,
and refetch/recover cleanly when the server reports a stale-version conflict.
