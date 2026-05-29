# HLS Progressive Runtime Boundary

Status: Active
Last updated: 2026-05-29

## Why This Lane Exists

Nako is targeting a future self-hosted media server comparable in playback
pressure to Jellyfin and Plex while keeping Nako's own Rust boundaries,
terminology, and FFmpeg CLI-first media-engine decision.

Recent HLS lanes made output shape, fMP4, adaptive ladders, selected subtitles,
alternate audio sidecars, master playlist authoring, and seek generation
identity executable. The remaining runtime risk is that the current HLS
execution path still waits for the whole FFmpeg process to complete before
publishing the playlist and final artifact directory.

That model blocks large media startup, undercuts running-segment readiness, and
will make future seek churn, adaptive playback, remote clients, and resource
scheduling harder than they need to be.

## Relevant Authority

- ADRs:
  - `docs/adr/0052-hls-runtime-and-media-engine-boundary.md`
  - `docs/adr/0053-application-control-plane-boundary.md`
  - `docs/adr/0049-source-aware-transcode-runtime.md`
  - `docs/adr/0045-ffmpeg-hardware-pipeline-planner.md`
  - `docs/adr/0038-playback-planning-and-transcode-policy-seams.md`
  - `docs/adr/0044-playback-capability-profile-planner.md`
- Architecture maps:
  - `docs/ARCHITECTURE.md`
  - `docs/architecture/PLAYBACK.md`
  - `docs/architecture/CONTROL_PLANE.md`
  - `docs/architecture/WORKSTREAM_LINKS.md`
- Related workstreams:
  - `docs/workstreams/source-aware-transcode-runtime/`
  - `docs/workstreams/transcode-output-shape-hls-manifest-ladder/`
  - `docs/workstreams/executable-hls-fmp4-runtime-boundary/`
  - `docs/workstreams/adaptive-hls-source-aware-ladder/`
  - `docs/workstreams/hls-media-renditions-runtime/`
  - `docs/workstreams/hls-master-renditions-authoring/`
  - `docs/workstreams/hls-audio-sidecar-artifacts/`
  - `docs/workstreams/hls-seek-restart-lifecycle/`

## Problem

Current HLS execution has several coupled assumptions:

- `FfmpegHlsRunner` rewrites all HLS outputs into a temporary directory and
  promotes that directory only after FFmpeg exits successfully.
- `HlsAppService` awaits the runner and returns a playlist only after the
  transcode session is marked finished.
- Running-session segment serving exists as a partial serving rule, but normal
  runner output is not visible at the final manifest path while the process is
  running.
- Server-side artifact reconstruction parses `request_key` strings to recover
  HLS output shape and request-variant identity.
- Playlist URL rewriting and browser/renderer ticket decoration are separate
  line-oriented passes.

These assumptions were reasonable during early VOD and safety slices. They are
now becoming architectural debt because HLS should behave like a runtime
session with manifest-backed artifact visibility, bounded readiness, and
explicit cancellation, not like a completed-file remux path with many segment
files.

## Target State

When this lane closes:

- HLS playlist requests start or reuse a transcode session without requiring
  the full media to finish transcoding before the playlist can be returned.
- Runtime artifact visibility is explicit: either FFmpeg writes to a
  serve-visible session directory guarded by a manifest and active marker, or a
  typed active artifact location is persisted and safely served.
- Segment routes can serve manifest-approved artifacts from running sessions
  and return bounded not-ready conflicts for artifacts that are not generated
  yet.
- HLS artifact reconstruction is owned by typed transcode/runtime data instead
  of server-local request-key substring parsing.
- Public playlist authoring, URL rewriting, and browser/renderer ticket
  decoration flow through one manifest-aware authoring boundary.
- Existing MPEG-TS, single-variant fMP4, adaptive fMP4, selected subtitle,
  audio sidecar, and seek-generation behavior remains covered by tests.

## In Scope

- HLS runner output publication policy and cleanup semantics.
- HLS app-service session start/reuse/admission changes needed for progressive
  playlist readiness.
- Manifest-backed serving of running HLS artifacts.
- Typed artifact reconstruction from transcode request/session data.
- Consolidation of playlist authoring and media URL auth decoration.
- Focused docs, tests, and evidence for the shipped runtime shape.

## Out Of Scope

- Replacing FFmpeg CLI with rsmpeg/libav.
- LL-HLS, DASH, CMAF encryption, DRM, and HLS key delivery.
- Remote transcode workers or distributed queues.
- Full GPU resource scheduler implementation.
- Web player seek UX, ABR controls, or client UI changes beyond preserving the
  existing public HLS route contract.
- Removing selected audio from the main video mux when audio sidecars exist.

## Starting Assumptions

| Assumption | Confidence | Evidence | Consequence if wrong |
| --- | --- | --- | --- |
| FFmpeg CLI remains the executable media engine for this lane. | High | ADR 0052. | Split a media-engine ADR before changing runner architecture. |
| The public HLS playlist and segment routes should remain compatible in the first slice. | High | Public Client route inventory and browser ticket workstreams. | Add a client-contract task before changing URLs. |
| Progressive HLS can be implemented without schema changes if artifact location remains deterministic. | Medium | Current request identity and staging slug model. | Add SQLite/PostgreSQL artifact manifest persistence as a bounded task. |
| Running artifact serving must not become directory listing. | High | ADR 0052 manifest-backed artifact rule. | Block the lane until typed artifact allow-listing is preserved. |
| Resource scheduling is related but not the first proof target. | Medium | PLAYBACK.md Lane E. | Split `playback-runtime-resource-scheduler` if concurrency policy becomes blocking. |

## Architecture Direction

Keep the FFmpeg CLI-first boundary:

```text
PlaybackPlanningRequest
  -> TranscodeProfile + HLS request variant identity
  -> HLS runtime artifact spec
  -> HLS session admission
  -> supervised FFmpeg process
  -> playlist readiness + manifest-backed segment serving
```

`nako-transcode` should own HLS output identity, artifact specs, request-variant
parsing, FFmpeg command planning, and runner output publication mechanics.

`nako-server` should own storage/input leases, playback/transcode session
admission, cancellation registry, route access checks, and response streaming.
It should consume typed HLS artifact specs instead of parsing transcode request
keys.

Playlist authoring should become a single boundary that consumes:

- `HlsArtifactManifest`;
- route URL construction;
- browser or renderer auth decoration;
- playback-session or transcode-session URL binding.

## Closeout Condition

This lane can close when:

- the current whole-output blocking HLS runtime assumption has been removed or
  explicitly isolated behind a tested compatibility mode;
- running HLS playlist/segment readiness is covered by focused tests;
- artifact reconstruction no longer relies on server-local substring parsing;
- playlist URL/auth decoration is consolidated or a narrower follow-on is
  explicitly opened;
- docs and architecture indexes reflect the shipped behavior;
- focused `nako-transcode` and `nako-server` gates pass with fresh evidence.
