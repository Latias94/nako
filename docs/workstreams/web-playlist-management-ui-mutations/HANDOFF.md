# Web Playlist Management UI Mutations - Handoff

Status: Closed
Last updated: 2026-05-29

## Current State

This lane is closed. WPMU-020 through WPMU-050 shipped Public Client-backed
playlist create, rename, delete, add item, remove item, and explicit up/down
reorder. WPMU-060 verified the lane with full web gates, desktop/mobile browser
smoke, workstream review, closeout notes, and residual follow-ons.

The shipped implementation keeps playlist management on the Public Client
data-source and TanStack Query hook boundary. Fixture mode reports
non-persistence instead of pretending writes succeeded.

## Active Task

None. `WPMU-060` is complete.

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

Return to `web-deferred-product-reentry-plan` or open a new follow-on lane for
the next playlist product investment. Good candidates are broader
add-to-playlist coverage, drag-and-drop reorder, shared playlists,
recommendation/smart lists, offline sync conflict handling, or future
Tauri/mobile playlist surfaces.
