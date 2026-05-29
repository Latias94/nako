# HLS Progressive Runtime Boundary — Handoff

Status: Completed
Last updated: 2026-05-29

## Current State

HPRB-060 is complete. `nako-transcode` has an explicit
`HlsOutputPublicationPolicy`: the default `AtomicOnCompletion` path preserves
the existing temporary-directory promotion behavior, and server HLS now uses
`ServeWhileRunning` so playlist-facing app and HTTP paths can observe playlists
and segments before FFmpeg exits.

The lane targets the HLS runtime boundary after the completed fMP4, adaptive
ladder, media rendition, audio sidecar, master playlist authoring, and seek
generation lanes. The progressive runtime behavior is now proven for the server
entry points, and artifact reconstruction now flows through
`nako-transcode::HlsArtifactSpec` instead of server-local `request_key`
substring parsing. Playlist authoring, session route binding, and browser or
renderer auth query decoration now flow through one manifest-aware app-layer
boundary. Closeout verification tightened running playlist readiness: the
server no longer treats a partially written playlist file as ready until it
contains at least one media or variant URI line.

## Active Task

- Task ID: none
- Owner: planner
- Files:
  - `docs/workstreams/hls-progressive-runtime-boundary`
  - `docs/architecture/PLAYBACK.md`
  - `docs/architecture/WORKSTREAM_LINKS.md`
- Validation:
  - `cargo nextest run -p nako-transcode hls --no-fail-fast`
  - `cargo nextest run -p nako-server hls --no-fail-fast`
  - `cargo nextest run -p nako-server playback --no-fail-fast`
  - `cargo fmt --all -- --check`
  - `git diff --check`
- Status: COMPLETED
- Review: final closeout gates passed
- Evidence: `docs/workstreams/hls-progressive-runtime-boundary/EVIDENCE_AND_GATES.md`

## Decisions Since Last Update

- Keep FFmpeg CLI as the executable media engine.
- Preserve existing public HLS playlist and segment route contracts in the
  first implementation slice.
- Do not combine LL-HLS, DASH, DRM, remote workers, or full resource scheduler
  work into this lane.
- Treat running artifact visibility as a manifest-backed runtime concern, not
  directory listing.
- Preserve atomic VOD output publication as the default transcode runner mode.
- Use `HlsOutputPublicationPolicy::ServeWhileRunning` for the progressive server
  proof so playlist readiness can be tested before full transcode completion.
- Prepare FFmpeg input before spawning the background HLS task so pre-session
  staging/input errors return to the playlist request instead of waiting for the
  readiness timeout.
- Reconstruct HLS artifact manifests through `nako-transcode::HlsArtifactSpec`
  from persisted transcode request identity. No schema migration was needed.
- Keep HLS playlist auth decoration in app playback playlist authoring rather
  than HTTP-local line-oriented rewrite helpers.
- Carry browser and renderer HLS transport query strings through playback app
  requests so the manifest-aware boundary can decorate route-bound playlist
  URIs in one pass.

## Blockers

- None for HPRB-060.

## Next Recommended Action

- Keep HLS progressive runtime in maintenance mode.
- Open separate lanes for LL-HLS, DASH/CMAF, DRM/key delivery, remote transcode
  workers, selected-main-audio cleanup, or the playback runtime resource
  scheduler when those become the next priority.
