# User Playlists Contract And Web Slice - Handoff

Status: Active
Last updated: 2026-05-29

## Current State

This lane is open. UPCW-020 froze the User Playlist Public Client contract.
Playlist UI remains blocked until backend persistence, app-service behavior,
Public Client routes, SDK methods, and access enforcement land in later tasks.

Important boundaries:

- User Playlist is not catalog Collection.
- User Playlist is not HLS `playlist.m3u8`.
- User Playlist is not global Canonical Metadata.
- Public routes should be current-user routes under `/users/me/playlists`.
- Public item responses omit inaccessible membership rows instead of returning
  tombstones.
- Duplicate playlist membership is rejected by contract; add is idempotent.
- Frontend work must use Public Client contracts, not Admin API.

## Active Task

- Task ID: UPCW-030
- Owner: Codex
- Status: READY
- Validation: `cargo nextest run -p nako-db playlist --no-fail-fast`; focused
  app-service tests; `cargo fmt --all -- --check`.

## Next Recommended Action

Start UPCW-030. Implement principal-scoped playlist records, ordered membership
persistence, and app-service validation without adding Public Client HTTP
routes or `web/` UI yet.
