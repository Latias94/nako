# HLS Progressive Runtime Boundary — Closeout

Status: Completed
Closed: 2026-05-29

## Outcome

HPRB closed the whole-output HLS runtime assumption for the current Nako server
slice. HLS playlist requests can now start or reuse a running transcode session,
return after bounded playlist readiness, serve manifest-approved generated
artifacts, and preserve browser/renderer ticket decoration through one
app-layer playlist authoring boundary.

The final closeout tightened readiness beyond simple path existence: a running
playlist is not ready until it contains at least one non-comment media or
variant URI line. This avoids returning partially written playlists while still
allowing progressive serving before FFmpeg exits.

## Shipped Shape

- `nako-transcode` owns `HlsOutputPublicationPolicy`, preserving atomic VOD
  promotion while exposing `ServeWhileRunning` for server HLS.
- `nako-transcode::HlsArtifactSpec` reconstructs HLS artifact manifests from
  persisted request identity.
- `nako-server` consumes typed HLS manifests for playlist authoring, segment
  allow-listing, browser ticket decoration, renderer ticket decoration, and
  playback-session route binding.
- Running sessions return bounded not-ready conflicts for manifest-approved
  artifacts that have not been generated yet.
- Public HLS playlist and segment routes remain compatible with the existing
  client contract.

## Final Gates

- `cargo nextest run -p nako-transcode hls --no-fail-fast`
- `cargo nextest run -p nako-server hls --no-fail-fast`
- `cargo nextest run -p nako-server playback --no-fail-fast`
- `cargo fmt --all -- --check`
- `git diff --check`

All final gates passed on 2026-05-29.

## Deferred Follow-Ons

- `proposed:playback-runtime-resource-scheduler`
- `proposed:ll-hls-cmaf-runtime`
- `proposed:dash-cmaf-playback-packaging`
- `proposed:hls-key-delivery-drm-boundary`
- `proposed:remote-transcode-worker-runtime`
- `proposed:hls-selected-main-audio-cleanup`
