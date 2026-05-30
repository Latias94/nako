# Playback Architecture

Last updated: 2026-05-30

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
| Direct Play byte ranges | Shipped | `docs/adr/0017-playback-streaming-and-remote-hardening-boundaries.md`; `docs/workstreams/playback-streaming/` | Client/player UX and remote transport polish. |
| Remux / Direct Stream | Shipped | `docs/workstreams/source-aware-transcode-runtime/`; `docs/adr/0049-source-aware-transcode-runtime.md` | Container-specific compatibility reasons and TV/device profiles. |
| Playback decision model | Shipped foundation | `docs/adr/0038-playback-planning-and-transcode-policy-seams.md`; `docs/adr/0044-playback-capability-profile-planner.md`; `docs/workstreams/playback-planner-transcode-seam-deepening/` | Richer device capability profiles and transcode seam cleanup. |
| Browser playback tickets | Shipped | `docs/adr/0036-short-lived-browser-playback-tickets.md`; `docs/workstreams/browser-playback-auth-transport/` | Player integration and cross-device resume polish. |
| Renderer transport tickets | Shipped | `docs/adr/0041-renderer-cast-safe-transport-tickets.md` | Chromecast/DLNA/AirPlay adapter lanes. |
| FFmpeg command planning | Shipped foundation | `docs/adr/0045-ffmpeg-hardware-pipeline-planner.md`; `docs/adr/0052-hls-runtime-and-media-engine-boundary.md` | Tone mapping, audio filters, seek restart commands. |
| Hardware detection and fallback | Partial | `docs/adr/0046-ffmpeg-probe-inventory.md`; `docs/adr/0047-cpu-transcode-readiness.md`; `docs/adr/0048-playback-transcode-startup-degradation.md` | GPU resource scheduler and per-host readiness diagnostics. |
| HLS single-variant MPEG-TS | Shipped | `docs/workstreams/transcode-output-shape-hls-manifest-ladder/` | Keep as compatibility baseline. |
| HLS single-variant fMP4 | Shipped | `docs/workstreams/executable-hls-fmp4-runtime-boundary/` | Player validation and browser compatibility matrix. |
| Adaptive HLS fMP4 ladder | Shipped first slice | `docs/workstreams/adaptive-hls-source-aware-ladder/` | Bandwidth-aware ABR and variant pruning. |
| HLS artifact manifest | Shipped | `docs/workstreams/transcode-output-shape-hls-manifest-ladder/`; `docs/adr/0052-hls-runtime-and-media-engine-boundary.md` | Keep all playlist/media group URLs manifest-backed. |
| Selected audio stream mapping | Shipped | `docs/workstreams/hls-alternate-audio-renditions/`; `docs/workstreams/hls-selected-main-audio-cleanup/`; `docs/workstreams/playback-audio-language-default-policy/` | Request-scoped language/default policy is shipped; persist user settings later. |
| HLS subtitle sidecar media group | Shipped first slice | `docs/workstreams/hls-media-renditions-runtime/`; `docs/workstreams/hls-master-renditions-authoring/`; `docs/workstreams/playback-subtitle-language-default-policy/` | Request-scoped subtitle language/default policy is shipped; defer ASS/SSA, PGS, burn-in, and client subtitle capability policy. |
| HLS audio sidecar media group | Shipped cleanup slice | `docs/workstreams/hls-audio-sidecar-artifacts/`; `docs/workstreams/hls-selected-main-audio-cleanup/`; `docs/workstreams/playback-audio-language-default-policy/` | Request-scoped language defaults are shipped; defer audio codec policy, downmix, and normalization. |
| HLS seek/restart | Shipped first slice | `docs/adr/0052-hls-runtime-and-media-engine-boundary.md`; `docs/workstreams/hls-seek-restart-lifecycle/` | Generation identity, restart admission, FFmpeg seek flags, and public `start_position_ms` playlist query. |
| HLS progressive runtime | Shipped | `docs/workstreams/hls-progressive-runtime-boundary/`; `docs/adr/0052-hls-runtime-and-media-engine-boundary.md` | Playlist readiness before full FFmpeg completion, running segment serving, typed artifact reconstruction, manifest-aware URL auth, and partial-playlist readiness guard. |
| HDR tone mapping | Draft research lane | `docs/ARCHITECTURE.md`; `docs/adr/0044-playback-capability-profile-planner.md`; `docs/workstreams/hdr-tone-mapping-pipeline/` | Run `HTP-010` docs/research before implementation. |
| Audio downmix and normalization | Active first slice | `docs/workstreams/audio-compatibility-downmix-normalization/` | Run `ACDN-020` in `nako-playback` before transcode propagation. |
| Runtime resource scheduler | Shipped first slice | `docs/workstreams/playback-runtime-resource-scheduler/`; `docs/adr/0005-bounded-async-pipelines-and-resource-budgets.md`; playback runtime diagnostics lanes | Add queueing, remote workers, OS isolation, per-device tuning, and disk-sensitive artifact I/O enforcement only through follow-on lanes. |
| VFS/remote playback resilience | Partial | `docs/adr/0016-remote-storage-and-vfs-cache-boundary.md`; `docs/adr/0017-playback-streaming-and-remote-hardening-boundaries.md` | Timeout/circuit-breaker and remote staging hardening. |
| SQLite/PostgreSQL write pressure | Good foundation | `docs/adr/0029-postgresql-ready-persistence-boundary.md`; `docs/adr/0030-postgresql-ready-sql-dialect-and-migration-policy.md`; PostgreSQL readiness lanes | Playback heartbeat/session-write pressure tests. |
| Release and packaging | Partial | `docs/deployment/SELF_HOSTED.md`; `docs/deployment/RELEASE_CHECKLIST.md`; `scripts/release-gate.*` | FFmpeg/hardware matrix packaging gate. |
| Web player integration | Partial | Media Web workstreams | HLS.js/Shaka integration and capability reporting. |

## Workstream Evidence

Use `docs/architecture/WORKSTREAM_LINKS.md#playback-and-transcode` as the
consolidated index for playback and transcode workstreams. Keep capability rows
linked to the most direct ADR/workstream evidence.

## Parallel Work Lanes

These lanes are intentionally separable. They can run in parallel if each lane
keeps its public contract explicit.

The current HDR terminal may run beside audio compatibility only for
`HTP-010`, which is docs/research-only. Do not implement HDR code while
`ACDN-020` is changing playback audio requirement vocabulary unless the planner
serializes the shared playback/transcode scopes.

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

Current status: draft. Run `HTP-010` to confirm the first executable slice and
validation before code changes.

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

Current status: active. Run `ACDN-020` before transcode or server changes.

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

- admission queueing and waitlist policy;
- remote transcode worker runtime;
- OS-level cgroups, process priority, and GPU vendor scheduling;
- per-device and per-host capacity tuning;
- disk-sensitive HLS artifact read/write pressure enforcement.

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
Future audio compatibility work should model:

- channel downmix;
- dynamic range compression;
- loudness normalization;
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
