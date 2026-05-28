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
WDRP-060 recorded `NON_VIDEO_DOMAIN_DECISION.md`; photos, music, and podcasts
remain deferred until a concrete ADR-0021 domain-baseline trigger appears.

## Active Task

- Task ID: WDRP-065
- Owner: planner
- Status: READY
- Validation: route WMLP closeout follow-ons into explicit workstream
  decisions.

## Next Recommended Action

- Start WDRP-065. Use WMLP closeout evidence to decide how to track browser
  playback session identity, library browse, catalog sort/filter, and desktop
  native playback.
