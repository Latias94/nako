# HLS Master Renditions Authoring Milestones

Status: Active
Last updated: 2026-05-29

## Milestone 1 - Workstream Opened

Status: Done

- Created durable docs for the HLS master rendition authoring lane.
- Split selected subtitle discoverability from alternate audio, image subtitle,
  LL-HLS, DRM, and second-engine adapter work.

## Milestone 2 - Typed Master Authoring Boundary

Status: Pending

- HLS master playlist authoring is derived from `HlsArtifactManifest` and media
  rendition plans.
- Subtitle media groups can be emitted without one-off playlist string hacks.

## Milestone 3 - Selected Subtitle Discoverability

Status: Pending

- Selected subtitle WebVTT sidecar playlists are advertised through standard
  `EXT-X-MEDIA` tags.
- Single-variant and adaptive HLS entry playlists expose subtitle groups.

## Milestone 4 - Runtime Verified And Closed

Status: Pending

- HLS source runtime, artifact serving, playlist rewrite, session reuse, and
  Public/Admin redaction gates pass.
- Workstream evidence is recorded and status is closed.
