# Playback Architecture

Last updated: 2026-06-04

This document is the agent-facing progress map for Nako video playback. It
links expected media-server capabilities to current implementation state,
authority docs, and the next parallel work lanes.

## Target Playback Chain

```text
MediaSource / Source Locator
  -> Storage/VFS read capability
  -> MediaProbeResult
  -> ClientPlaybackCapabilities + policy + user preferences
  -> PlaybackRenditionPlan
  -> Direct Play | Remux | HLS Transcode
  -> Transcode/Playback Session
  -> Artifact Manifest
  -> ticketed Public/Renderer transport
  -> client player + heartbeat/progress
```

The product rule is: prefer Direct Play, then Remux, then Transcode. A
transcode must be explainable from source facts, client facts, policy, or user
selection.

## Capability Progress Matrix

| Capability | Status | Authority | Next Lane |
| --- | --- | --- | --- |
| Direct Play byte ranges | Shipped with no-store cache baseline | `docs/adr/0017-playback-streaming-and-remote-hardening-boundaries.md`; `docs/workstreams/playback-streaming/` | Client/player UX and remote transport polish. |
| Remux / Direct Stream | Shipped with no-store cache baseline and `app/playback/remux_flow.rs` lifecycle boundary | `docs/workstreams/source-aware-transcode-runtime/`; `docs/adr/0049-source-aware-transcode-runtime.md`; `.trellis/spec/nako-server/backend/directory-structure.md#scenario-playback-remux-lifecycle-orchestration` | Container-specific compatibility reasons and TV/device profiles. |
| Playback decision model | Shipped with matrix coverage | `docs/adr/0038-playback-planning-and-transcode-policy-seams.md`; `docs/adr/0044-playback-capability-profile-planner.md`; `docs/workstreams/playback-planner-transcode-seam-deepening/`; `docs/workstreams/playback-compatibility-matrix-hardening/` | Split exhaustive device profile matrices, API reporting, or player controls into follow-ons. |
| Playback-to-transcode Interface | Shipped deeper Interface slice | `docs/adr/0038-playback-planning-and-transcode-policy-seams.md`; `docs/adr/0045-ffmpeg-hardware-pipeline-planner.md`; `docs/workstreams/transcode-interface-and-runtime-plan-deepening/` | Extend the transcode-owned runtime/execution planners in HDR and future filter lanes; do not reintroduce server-side raw FFmpeg request assembly. |
| Browser playback tickets | Shipped | `docs/adr/0036-short-lived-browser-playback-tickets.md`; `docs/workstreams/browser-playback-auth-transport/` | Player integration and cross-device resume polish. |
| Renderer transport tickets | Shipped with `app/playback/renderer_flow.rs` playback-session boundary | `docs/adr/0041-renderer-cast-safe-transport-tickets.md`; `.trellis/spec/nako-server/backend/directory-structure.md#scenario-playback-renderer-transport-flow-orchestration` | Chromecast/DLNA/AirPlay adapter lanes. |
| FFmpeg command planning | Shipped foundation | `docs/adr/0045-ffmpeg-hardware-pipeline-planner.md`; `docs/adr/0052-hls-runtime-and-media-engine-boundary.md` | Tone mapping, audio filters, seek restart commands. |
| Hardware detection and fallback | Shipped broader inventory evidence plus HLS output policy seam | `docs/adr/0046-ffmpeg-probe-inventory.md`; `docs/adr/0047-cpu-transcode-readiness.md`; `docs/adr/0048-playback-transcode-startup-degradation.md`; `docs/workstreams/transcode-capability-inventory-matrix/`; `.trellis/tasks/06-04-06-04-hevc-av1-hls-output-policy-first-slice/` | Split hardware tone-map execution, HEVC/AV1 FFmpeg execution, Admin/release reporting, and hardware smoke into follow-ons. |
| HLS single-variant MPEG-TS | Shipped | `docs/workstreams/transcode-output-shape-hls-manifest-ladder/` | Keep as compatibility baseline. |
| HLS single-variant fMP4 | Shipped | `docs/workstreams/executable-hls-fmp4-runtime-boundary/` | Player validation and browser compatibility matrix. |
| Adaptive HLS fMP4 ladder | Shipped first slice | `docs/workstreams/adaptive-hls-source-aware-ladder/` | Bandwidth-aware ABR and variant pruning. |
| HLS artifact manifest | Shipped | `docs/workstreams/transcode-output-shape-hls-manifest-ladder/`; `docs/adr/0052-hls-runtime-and-media-engine-boundary.md` | Keep all playlist/media group URLs manifest-backed. |
| Selected audio stream mapping | Shipped | `docs/workstreams/hls-alternate-audio-renditions/`; `docs/workstreams/hls-selected-main-audio-cleanup/`; `docs/workstreams/playback-audio-language-default-policy/` | Request-scoped language/default policy is shipped; persist user settings later. |
| HLS subtitle sidecar / burn-in planning | Shipped sidecar plus burn-in planning first slice | `docs/workstreams/hls-media-renditions-runtime/`; `docs/workstreams/hls-master-renditions-authoring/`; `docs/workstreams/playback-subtitle-language-default-policy/`; `.trellis/tasks/06-04-hls-subtitle-burn-in-planning/` | Request-scoped subtitle language/default policy is shipped. Sidecar-capable selected text subtitles remain WebVTT sidecars; known non-sidecar formats now emit typed burn-in intent and block Remux fallback. Missing or blank subtitle codec facts intentionally preserve legacy sidecar behavior until richer client subtitle capability profiles land. Image-subtitle execution remains a follow-on. |
| HLS audio sidecar media group | Shipped cleanup slice | `docs/workstreams/hls-audio-sidecar-artifacts/`; `docs/workstreams/hls-selected-main-audio-cleanup/`; `docs/workstreams/playback-audio-language-default-policy/` | Request-scoped language defaults and audio output compatibility are shipped; defer codec-aware sidecars and player-specific fallback. |
| HLS seek/restart | Shipped first slice | `docs/adr/0052-hls-runtime-and-media-engine-boundary.md`; `docs/workstreams/hls-seek-restart-lifecycle/` | Generation identity, restart admission, FFmpeg seek flags, and public `start_position_ms` playlist query. |
| HLS progressive runtime | Shipped | `docs/workstreams/hls-progressive-runtime-boundary/`; `docs/adr/0052-hls-runtime-and-media-engine-boundary.md` | Playlist readiness before full FFmpeg completion, running segment serving, typed artifact reconstruction, manifest-aware URL auth, and partial-playlist readiness guard. |
| HLS runtime lifecycle | Closed with test-stability follow-on | `docs/adr/0052-hls-runtime-and-media-engine-boundary.md`; `docs/workstreams/hls-runtime-lifecycle-boundary/`; `docs/workstreams/hls-progressive-readiness-test-stability/` | Lifecycle invariants are frozen, behavior-preserving tests are in place, and HPRTS stabilized the full HLS gate. Ordinary HLS startup now uses a resource-owned bounded `HlsStart` policy before FFmpeg input staging, while replacement flows keep `HlsSupersede`; durable queueing, remote workers, LL-HLS/CMAF, and player UX remain follow-ons. |
| HDR tone mapping | Shipped software-first slice | `docs/ARCHITECTURE.md`; `docs/adr/0044-playback-capability-profile-planner.md`; `docs/workstreams/hdr-tone-mapping-pipeline/` | Split hardware tone mapping, dynamic HDR handling, device profiles, UI controls, and operator smoke matrices into follow-ons. |
| Audio downmix and normalization | Shipped first slice | `docs/workstreams/audio-compatibility-downmix-normalization/` | Persisted preferences, client controls, device profiles, and dialogue clarity remain follow-ons. |
| Runtime resource scheduler | Shipped first slice plus bounded HLS start admission | `docs/workstreams/playback-runtime-resource-scheduler/`; `docs/adr/0005-bounded-async-pipelines-and-resource-budgets.md`; playback runtime diagnostics lanes | HLS artifact I/O session admission and ordinary HLS start waits are enforced through typed playback resource permits. Add durable queueing, remote workers, OS isolation, per-device tuning, and per-artifact read/write pressure policy only through follow-on lanes. |
| VFS/remote playback resilience | Partial | `docs/adr/0016-remote-storage-and-vfs-cache-boundary.md`; `docs/adr/0017-playback-streaming-and-remote-hardening-boundaries.md` | Timeout/circuit-breaker and remote staging hardening. |
| SQLite/PostgreSQL write pressure | Good foundation | `docs/adr/0029-postgresql-ready-persistence-boundary.md`; `docs/adr/0030-postgresql-ready-sql-dialect-and-migration-policy.md`; PostgreSQL readiness lanes | Playback heartbeat/session-write pressure tests. |
| Release and packaging | Partial | `docs/deployment/SELF_HOSTED.md`; `docs/deployment/RELEASE_CHECKLIST.md`; `scripts/release-gate.*` | FFmpeg/hardware matrix packaging gate. |
| Web player integration | Shipped first slice | Media Web workstreams | Browser HLS playback now prefers native support and lazy-loads `hls.js` fallback while preserving Direct Play and ticket redaction. Follow with capability reporting, richer retry UX, and desktop/native player decisions. |

## Workstream Evidence

Use `docs/architecture/WORKSTREAM_LINKS.md#playback-and-transcode` as the
consolidated index for playback and transcode workstreams. Keep capability rows
linked to the most direct ADR/workstream evidence.

## Parallel Work Lanes

These lanes are intentionally separable. They can run in parallel if each lane
keeps its public contract explicit.

The audio compatibility, Transcode Interface deepening, software-first HDR
tone-mapping, playback compatibility matrix, and transcode capability
inventory slices are closed. `hls-runtime-lifecycle-boundary` completed its
docs/research invariant freeze, behavior-preserving lifecycle coverage slice,
follow-on split decisions, and closeout retry. `hls-progressive-readiness-test-stability`
closed the full-suite progressive-readiness gate instability that blocked HRLB
closeout. `playback-transcode-jellyfin-class-hardening` is closed after
freezing and implementing the parallel Playback Capability, Transcode Pipeline
Capability, FFmpeg Adapter, HLS Artifact Authority, and Playback Runtime
slices. Playback Runtime supersede ownership now covers HLS candidate
discovery, cancellation, bounded replacement admission, playback-session
cancellation after supersede, and first HLS artifact I/O session admission.
Keep resource admission queueing, per-artifact read/write pressure, remote
workers, LL-HLS/CMAF, and player-facing follow-ons separate.

The playback resource admission first bounded-wait slice is process-local:
ordinary HLS source and playlist startup use `HlsStart` before FFmpeg input
staging, supersede continues to use `HlsSupersede`, and Direct Play remote
stream admission remains non-blocking.

Remux lifecycle orchestration is now a server-side app boundary in
`crates/nako-server/src/app/playback/remux_flow.rs`: the playback app root keeps
thin Remux entry points, while source context construction, immediate resource
admission, background start, playback-session linkage, FFmpeg input
staging/release, active/completed session reuse, and output waiting stay in the
focused flow module. Keep public API, DTO, schema, and generated SDK shape
unchanged for this boundary.

Renderer playback transport orchestration is now a server-side app boundary in
`crates/nako-server/src/app/playback/renderer_flow.rs`: the playback app root
keeps a thin renderer entry point, while source/probe context, `RemoteControl`
policy enforcement, playback decision planning, Direct/Remux/HLS session
startup, transcode linkage, HLS supersede cleanup, and renderer transport plan
construction stay in the focused flow module. Renderer ticket issuance and URL
authoring remain in `http/renderer.rs`; public API, DTO, schema, and generated
SDK shape stay unchanged.

The HEVC/AV1 HLS output policy first slice recognizes H264, HEVC/H265, and AV1
as typed profile policy values while keeping H264/AAC as the only executable
HLS output. HEVC/AV1 FFmpeg encoder argv, client compatibility, and hardware
selection remain follow-ons.

### Lane A - Device Capability Profiles

Goal: Make clients report enough codec/container/subtitle/HDR/audio facts for
the planner to choose Direct Play, Remux, or Transcode accurately.

Primary crates and docs:

- `crates/nako-playback`
- `crates/nako-api`
- `crates/nako-server`
- `docs/adr/0044-playback-capability-profile-planner.md`

Exit criteria:

- browser/mobile/TV/renderer capability records exist;
- planner emits precise compatibility reasons;
- public DTOs remain redaction-safe.

### Lane B - HLS Seek / Restart Lifecycle

Goal: Support seek-aware HLS session restart without corrupt playback,
leaking stale artifacts, or confusing session reuse.

Primary crates and docs:

- `crates/nako-transcode`
- `crates/nako-server/src/app/playback`
- `docs/adr/0052-hls-runtime-and-media-engine-boundary.md`

Exit criteria:

- seek requests carry stable identity;
- old FFmpeg sessions are cancelled or superseded deterministically;
- segment routes serve only the active manifest window;
- keyframe alignment and timestamp behavior are tested.

### Lane B2 - HLS Progressive Runtime Boundary

Status: Completed.

Goal: Make HLS behave like a runtime session with manifest-backed artifact
visibility instead of a whole-output materialization path.

Primary crates and docs:

- `crates/nako-transcode`
- `crates/nako-server/src/app/playback`
- `docs/adr/0052-hls-runtime-and-media-engine-boundary.md`
- `docs/workstreams/hls-progressive-runtime-boundary/`

Exit criteria:

- playlist readiness no longer requires full FFmpeg completion;
- running segment routes serve only manifest-approved artifacts;
- artifact reconstruction is typed rather than server-local request-key
  substring parsing;
- browser and renderer HLS auth decoration remains redaction-safe.

Follow-ons stay separate: LL-HLS/CMAF, DASH/CMAF, DRM/key delivery, remote
transcode workers, and full playback runtime resource scheduling. Selected
main audio cleanup is closed in
`docs/workstreams/hls-selected-main-audio-cleanup/`.

### Lane C - Subtitle Compatibility

Goal: Make subtitle selection reliable for SRT, WebVTT, ASS/SSA, and future
image subtitles.

Primary crates and docs:

- `crates/nako-transcode`
- `crates/nako-server`
- subtitle import and HLS rendition workstreams

Exit criteria:

- selected text subtitles can become sidecars when the client can render them;
- ASS/SSA and unsupported subtitle formats can trigger burn-in when needed;
- addon-provided subtitles have a bounded readiness policy.

Current status: first burn-in planning slice is shipped for HLS. Playback owns
the sidecar-versus-burn-in subtitle intent, transcode carries that intent
through HLS runtime identity, and the FFmpeg adapter accepts only embedded text
subtitle burn-in on the software pipeline. A selected known burn-in-only
subtitle blocks Direct Play and Remux before HLS transcode is planned. Missing
or blank subtitle codec facts keep the legacy sidecar path by explicit policy,
not by fallback accident. PGS/image subtitle burn-in execution, external
subtitle burn-in, hardware-filter burn-in, and richer client subtitle format
profiles remain follow-ons.

### Lane D - HDR / Tone Mapping

Goal: Make HDR media watchable on SDR clients with explicit color pipeline
planning.

Primary crates and docs:

- `crates/nako-playback`
- `crates/nako-transcode`
- `docs/adr/0045-ffmpeg-hardware-pipeline-planner.md`
- `docs/workstreams/hdr-tone-mapping-pipeline/`

Exit criteria:

- probe facts expose HDR/color compatibility inputs;
- planner selects direct/remux/transcode from client HDR capability;
- FFmpeg software and hardware tone mapping strategies are testable.

Current status: closed first slice. `HTP-020` shipped playback-owned **Color
Pipeline Requirement** vocabulary, and `HTP-030` shipped software-first HLS
HDR-to-SDR media output through the transcode-owned runtime/execution planner
Interfaces. Keep hardware tone mapping, device-specific filter chains, Dolby
Vision dynamic handling, HDR10+ preservation, device profile databases, UI
controls, and operator smoke matrices as follow-ons unless split explicitly.

### Lane F - Audio Compatibility

Goal: Make selected audio playable on constrained clients through explicit
channel, codec, downmix, dynamic range, and normalization requirements.

Primary crates and docs:

- `crates/nako-playback`
- `crates/nako-transcode`
- `crates/nako-server/src/app/playback`
- `docs/workstreams/audio-compatibility-downmix-normalization/`

Exit criteria:

- playback owns audio output requirements and compatibility reasons;
- transcode policy receives requirements without rebuilding playback decisions;
- FFmpeg command planning can emit deterministic downmix and normalization
  filters;
- HLS selected main audio and audio sidecar behavior stay compatible.

Current status: completed first slice. Follow-ons for persisted preferences,
client controls, device profile databases, and dialogue clarity should open as
separate workstreams.

### Lane E - Runtime Resource Scheduler

Status: Completed first slice.

Goal: Prevent playback workloads from exhausting CPU, GPU, disk, DB, or async
runtime capacity.

Primary crates and docs:

- runtime supervisor modules;
- `crates/nako-transcode`;
- `crates/nako-server/src/app/playback`;
- `docs/workstreams/playback-runtime-resource-scheduler/`;
- `docs/adr/0005-bounded-async-pipelines-and-resource-budgets.md`.

Exit criteria:

- playback resource demand is typed before process-backed work starts;
- HLS/remux start paths acquire explicit CPU/GPU/remux permits;
- active session reuse avoids double-acquiring permits;
- Admin diagnostics expose configured capacity and current pressure without
  leaking local paths, locators, filenames, or command lines.

Follow-ons:

- HLS progressive-readiness test stability;
- admission queueing and waitlist policy;
- remote transcode worker runtime;
- OS-level cgroups, process priority, and GPU vendor scheduling;
- per-device and per-host capacity tuning;
- per-artifact HLS read/write queueing and waitlist policy.

## Risk Register

### HLS Seek Requires Keyframe And Timestamp Discipline

Seeking with `-ss` is not a string-substitution task. Fast seek before `-i` can
land near a keyframe but may produce imperfect startup frames. Accurate seek
after `-i` costs more CPU. HLS segments also need keyframe-aligned boundaries.

Future seek work must decide:

- fast seek, accurate seek, or hybrid seek per source;
- GOP/keyframe policy for transcoded outputs;
- whether to use timestamp-preservation flags such as `-copyts`;
- how playlist sequence numbers and segment names remain stable after restart.

### Addon Subtitle Readiness Must Be Bounded

Playback must not block indefinitely on addon subtitle scraping. A future addon
readiness hook should be a bounded pre-play window or an explicit background
prefetch/import path.

Good behavior:

- use local/known subtitles immediately;
- wait only within a small configured budget for addon-produced sidecars;
- surface late subtitles as a refreshable track list rather than failing
  playback.

### Tokio Runtime Must Be Protected From Slow Media I/O

FFmpeg should run through `tokio::process`. Async file serving should stay on
async/tower primitives where possible. Use `spawn_blocking` for truly blocking
filesystem or library calls, not as a blanket wrapper around all media work.

Follow-on resource scheduling must still account for:

- slow disks and network mounts;
- concurrent segment writes and reads;
- remote staging;
- cleanup scans;
- DB writes from heartbeat/session metrics.

### Audio Compatibility Is More Than Codec Conversion

7.1 TrueHD/DTS-HD sources on 2.0 clients need policy beyond `-c:a aac`.
The closed audio compatibility first slice now models channel downmix and
loudness normalization. Future audio compatibility follow-ons should model:

- dynamic range compression;
- dialogue clarity;
- per-client night-mode or normalization preferences.

### VFS And Remote Mounts Can Hang

NAS, SMB, WebDAV, and rclone-like mounts can sleep, stall, or disconnect.
Playback, probe, scan, and metadata import must treat storage as fallible and
bounded.

Future VFS hardening should cover:

- read/probe/stage timeouts;
- circuit-breaker or backoff behavior;
- stale cache semantics;
- partial staging cleanup;
- operator diagnostics.

### SQLite Writes Need Playback Pressure Tests

SQLite is a good self-hosted default, but playback creates frequent writes:
heartbeats, session state, metrics, cleanup, and scan/provider jobs. Existing
SQLite/PostgreSQL boundaries are strong, but playback-specific pressure tests
should verify WAL behavior, busy timeouts, pool sizing, and transaction scope.

### Release Packaging Is A Playback Feature

Hardware acceleration only matters if the shipped environment exposes FFmpeg,
drivers, devices, and container permissions correctly. Release gates should
eventually validate:

- FFmpeg/ffprobe presence;
- VAAPI/NVENC/QSV/AMF/VideoToolbox diagnostics where available;
- Docker device documentation;
- smoke playback with CPU fallback;
- clear operator errors when hardware acceleration is unavailable.

## Agent Usage

Before opening a playback workstream:

1. Find the capability row in this document.
2. Read the linked ADR and latest workstream evidence.
3. Add or update an ADR only when the lane changes a durable boundary.
4. Keep task validation focused: planner tests, command-plan tests, server HLS
   tests, playback route tests, and redaction tests.
