# User Playlists Contract And Web Slice - Handoff

Status: Closed
Last updated: 2026-05-29

## Current State

This lane is closed. It delivered the User Playlist contract, persistence,
Public Client HTTP routes, OpenAPI/SDK coverage, Rust client methods,
access-filtered item responses/counts, and the first `web/` playlist UI slice
through Public Client live data, fixture fallback, and route-owned state.

Important boundaries:

- User Playlist is not catalog Collection.
- User Playlist is not HLS `playlist.m3u8`.
- User Playlist is not global Canonical Metadata.
- Public routes should be current-user routes under `/users/me/playlists`.
- Public item responses omit inaccessible membership rows instead of returning
  tombstones.
- Public playlist `item_count` is the current access-filtered count.
- Duplicate playlist membership is rejected by contract; add is idempotent.
- Backend membership persistence now keeps one row per playlist/media item and
  preserves explicit zero-based order.
- Frontend work must use Public Client contracts, not Admin API.

## Closed Task

- Task ID: UPCW-060
- Owner: planner
- Status: DONE
- Validation: final backend/frontend gates recorded; `WORKSTREAM.json`
  validation; `git diff --check`.

## Follow-Ons

- Web playlist management UI: create, rename, delete, add item, remove item,
  and reorder controls.
- Shared/public playlists, invites, and collaboration.
- Smart playlists and recommendation-generated lists.
- Offline sync and conflict resolution.
- Playlist-aware mobile/Tauri surfaces once the web management UX settles.
