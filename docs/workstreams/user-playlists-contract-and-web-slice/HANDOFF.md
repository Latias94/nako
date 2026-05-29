# User Playlists Contract And Web Slice - Handoff

Status: Active
Last updated: 2026-05-29

## Current State

This lane is open. UPCW-020 froze the User Playlist Public Client contract,
UPCW-030 implemented backend persistence plus app-service validation, and
UPCW-040 exposed the contract through Public Client HTTP routes, access
filtering, and Rust client methods. Playlist UI can now start in UPCW-050.

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

- Task ID: UPCW-050
- Owner: Codex
- Status: READY
- Validation: `npm --prefix web run test`;
  `npm --prefix web run check`; `npm --prefix web run build:budget`;
  browser smoke.

## Next Recommended Action

Start UPCW-050. Restore the first playlist UI in `web/` using live Public
Client data with fixture fallback and route-owned state. Keep the UI on Public
Client contracts and avoid Admin API imports.
