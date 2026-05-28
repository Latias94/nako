# Web Deferred Product Reentry Plan - Handoff

Status: Active
Last updated: 2026-05-28

## Current State

WBBP closed with deferred surfaces removed from the live runtime. This lane now
owns the follow-on routing decision for those surfaces. The reentry matrix maps
downloads to Admin Acquisition Intake, AI/automation to Generated Artifacts and
Admin automation diagnostics, playlists to a future user-owned contract, and
photos/music/podcasts to a future non-video media-domain baseline.
WDRP-020 opened and completed
`docs/workstreams/web-media-live-public-client-parity` for the video-first Media
implementation lane. WDRP-030 opened
`docs/workstreams/web-admin-acquisition-intake` for the new `web/` Admin
Acquisition Intake route. WDRP-040 opened
`docs/workstreams/web-admin-generated-artifacts-automation` for the new `web/`
Admin Generated Artifacts / Automation route.
WDRP-050 opened `docs/workstreams/user-playlists-contract-and-web-slice` for
the playlist backend/Public Client contract. Playlist UI remains blocked until
that lane freezes route and DTO shape.

## Active Task

- Task ID: WDRP-060
- Owner: planner
- Status: READY
- Validation: decide when photos, music, and podcasts deserve a non-video
  media-domain baseline lane.

## Next Recommended Action

- Start WDRP-060. Reference ADR-0021 and avoid UI-first non-video surfaces.
