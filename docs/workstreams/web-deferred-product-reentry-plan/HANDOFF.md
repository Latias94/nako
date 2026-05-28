# Web Deferred Product Reentry Plan - Handoff

Status: Completed
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
WDRP-065 recorded `PUBLIC_CLIENT_FOLLOW_ONS.md`, opened
`docs/workstreams/public-client-browser-playback-session-identity`, opened
`docs/workstreams/public-client-library-browse-query-contract`, and kept desktop
native playback deferred to the existing Rust/Tauri capability gap.

## Active Task

- Task ID: closed
- Owner: planner
- Status: DONE
- Validation: WDRP closeout is complete.

## Next Recommended Action

- Continue a selected follow-on lane:
  `public-client-library-browse-query-contract` at PLBQ-020,
  `public-client-browser-playback-session-identity` at PBSI-020,
  `web-admin-acquisition-intake` at WAAI-020,
  `web-admin-generated-artifacts-automation` at WAGA-020, or
  `user-playlists-contract-and-web-slice` at UPCW-020.
