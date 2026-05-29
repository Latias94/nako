# User Playlists Contract And Web Slice - Handoff

Status: Active
Last updated: 2026-05-28

## Current State

This lane is open. WDRP-050 decided that playlists are ready for a
backend/Public Client contract lane because User Playback State and effective
Library Access are already implemented, but playlist UI remains blocked until
UPCW-020 freezes the public contract.

Important boundaries:

- User Playlist is not catalog Collection.
- User Playlist is not HLS `playlist.m3u8`.
- User Playlist is not global Canonical Metadata.
- Public routes should be current-user routes under `/users/me/playlists`.
- Frontend work must use Public Client contracts, not Admin API.

## Active Task

- Task ID: UPCW-020
- Owner: Codex
- Status: READY
- Validation: contract docs, protocol/API tests when code changes, formatting,
  and diff check.

## Next Recommended Action

Start UPCW-020. Freeze route/DTO shape, access filtering, duplicate membership,
ordering, and SDK expectations before any `web/` playlist implementation.
