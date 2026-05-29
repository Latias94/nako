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
WPMU-050 added explicit up/down playlist item reorder controls with
stale-version conflict recovery.

The remaining lane work is verification and closeout. It must keep playlist
management on the Public Client boundary and must not import Admin API code
into media features.

## Active Task

- Task ID: WPMU-060
- Owner: Codex
- Files: `docs/workstreams/web-playlist-management-ui-mutations`
- Validation: `npm --prefix web run test`; `npm --prefix web run check`; `npm --prefix web run build:budget`; browser smoke desktop/mobile; `git diff --check`
- Status: READY
- Review: review-workstream has no blocking findings.
- Evidence: closeout notes, final gate results, and residual follow-ons

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
- Reorder submits full `item_ids` plus `expected_version`; stale-version
  conflict recovery refetches the current playlist item order.

## Blockers

- None known.

## Next Recommended Action

Start WPMU-060: run the full lane gates, smoke desktop/mobile playlist
management flows, record closeout evidence, and split any residual follow-ons
before closing the workstream.
