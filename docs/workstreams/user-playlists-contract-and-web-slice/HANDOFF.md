# User Playlists Contract And Web Slice - Handoff

Status: Active
Last updated: 2026-05-29

## Current State

This lane is open. UPCW-020 froze the User Playlist Public Client contract, and
UPCW-030 implemented backend persistence plus app-service validation. Playlist
UI remains blocked until Public Client routes, SDK methods, and access
enforcement land in later tasks.

Important boundaries:

- User Playlist is not catalog Collection.
- User Playlist is not HLS `playlist.m3u8`.
- User Playlist is not global Canonical Metadata.
- Public routes should be current-user routes under `/users/me/playlists`.
- Public item responses omit inaccessible membership rows instead of returning
  tombstones.
- Duplicate playlist membership is rejected by contract; add is idempotent.
- Backend membership persistence now keeps one row per playlist/media item and
  preserves explicit zero-based order.
- Frontend work must use Public Client contracts, not Admin API.

## Active Task

- Task ID: UPCW-040
- Owner: Codex
- Status: READY
- Validation: focused API/server route tests; SDK generation check;
  `cargo nextest run -p nako-api playlist --no-fail-fast`;
  `cargo nextest run -p nako-server user_playlist --no-fail-fast`.

## Next Recommended Action

Start UPCW-040. Expose `/users/me/playlists` through Public Client HTTP routes,
map app-service records into public DTOs, enforce effective Library Access for
item responses, and add Rust client support. Do not restore `web/` playlist UI
until UPCW-040 is complete.
