# User Playlists Contract And Web Slice - Handoff

Status: Active
Last updated: 2026-05-29

## Current State

This lane is open. UPCW-020 froze the User Playlist Public Client contract,
UPCW-030 implemented backend persistence plus app-service validation, and
UPCW-040 exposed the contract through Public Client HTTP routes, access
filtering, and Rust client methods. UPCW-050 restored the first `web/`
playlist UI through Public Client live data, fixture fallback, and route-owned
state.

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

## Active Task

- Task ID: UPCW-060
- Owner: planner
- Status: READY
- Validation: final backend/frontend gates recorded; JSON validation;
  `git diff --check`.

## Next Recommended Action

Start UPCW-060. Close the lane with backend/API/SDK/web evidence and split
follow-ons for sharing, smart playlists, recommendation-generated lists, and
offline sync.
