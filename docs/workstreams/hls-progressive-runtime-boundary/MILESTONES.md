# HLS Progressive Runtime Boundary — Milestones

Status: Completed
Last updated: 2026-05-29

## M0 — Scope And Evidence Freeze

Status: Completed

Exit criteria:

- Problem and target state are explicit.
- Non-goals are explicit.
- Relevant ADRs/docs/workstreams are linked.
- First proof target is chosen.

Primary evidence:

- `docs/workstreams/hls-progressive-runtime-boundary/DESIGN.md`
- `docs/workstreams/hls-progressive-runtime-boundary/TODO.md`

## M1 — Runtime Publication Proof

Status: Completed

Exit criteria:

- HLS output publication behavior is explicit in `nako-transcode`.
- Current atomic VOD behavior is preserved by tests.
- A serve-visible running-output proof exists for server integration.

Primary gates:

- `cargo nextest run -p nako-transcode hls_runner --no-fail-fast`
- `cargo nextest run -p nako-transcode hls --no-fail-fast`

Evidence:

- `HlsOutputPublicationPolicy::AtomicOnCompletion` preserves the existing
  temporary-directory promotion path.
- `HlsOutputPublicationPolicy::ServeWhileRunning` proves serve-visible playlist
  and segment output while FFmpeg is still running.
- Cancel and failure paths remove the selected publication directory.

## M2 — Progressive Server Integration

Status: Completed

Exit criteria:

- HLS playlist requests can return after readiness without waiting for full
  transcode completion.
- Segment routes can serve manifest-approved running artifacts.
- Missing running segments return bounded not-ready errors.
- Cancellation and failure cleanup remain safe.

Primary gates:

- `cargo nextest run -p nako-server hls_source --no-fail-fast`
- `cargo nextest run -p nako-server hls_segment --no-fail-fast`
- `cargo nextest run -p nako-server hls --no-fail-fast`

Evidence:

- Playlist-facing app and HTTP paths return after the HLS playlist exists while
  the linked transcode session remains `Running`.
- Running sessions serve generated manifest-approved segments.
- Missing running segments return bounded not-ready conflicts.
- Playback-session cancellation signals the running HLS process and persists
  `Cancelled`.

## M3 — Manifest And Playlist Boundary Cleanup

Status: Completed

Exit criteria:

- Server-local request-key parsing is deleted or reduced to a typed adapter
  call owned by `nako-transcode`.
- Playlist authoring and auth decoration are handled by one manifest-aware
  boundary or a narrower follow-on is opened.
- Public Client and renderer HLS URL contracts remain stable.

Primary gates:

- `cargo nextest run -p nako-transcode hls_request_variant --no-fail-fast`
- `cargo nextest run -p nako-server hls_artifact --no-fail-fast`
- `cargo nextest run -p nako-server renderer --no-fail-fast`

Evidence:

- HLS artifact reconstruction now goes through `nako-transcode::HlsArtifactSpec`
  from persisted request identity.
- Server-local HLS `request_key` substring parsing has been removed from
  `hls_artifact.rs`.
- Playlist authoring, session route binding, and browser/renderer auth query
  decoration now flow through one manifest-aware app-layer boundary.

## M4 — Closeout

Status: Completed

Exit criteria:

- Fresh focused gate evidence is recorded.
- Architecture docs reflect the shipped runtime behavior.
- Remaining work is completed, deferred, or split into named follow-ons.
- `WORKSTREAM.json` status is updated.

Evidence:

- `cargo nextest run -p nako-transcode hls --no-fail-fast`
- `cargo nextest run -p nako-server hls --no-fail-fast`
- `cargo nextest run -p nako-server playback --no-fail-fast`
- `cargo fmt --all -- --check`
- `git diff --check`
- `docs/workstreams/hls-progressive-runtime-boundary/CLOSEOUT.md`
