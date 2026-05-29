# HLS Progressive Runtime Boundary — Handoff

Status: Active
Last updated: 2026-05-29

## Current State

HPRB-050 is complete. `nako-transcode` has an explicit
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
boundary. The remaining work is closeout verification and follow-on splitting.

## Active Task

- Task ID: HPRB-060
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
- Status: PENDING
- Review: pending
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

- Run `run-workstream-task` for HPRB-060.
- Execute the final closeout gates, update architecture docs only if they need
  a shipped-behavior refresh, and split LL-HLS, DASH, DRM, remote workers, or
  resource scheduler work into separate lanes instead of expanding this one.
