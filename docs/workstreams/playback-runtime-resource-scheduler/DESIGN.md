# Playback Runtime Resource Scheduler

Status: Completed
Last updated: 2026-05-29

## Why This Lane Exists

Nako is becoming a real self-hosted playback server. HLS now behaves like a
runtime session: playlist-facing routes can return while FFmpeg is still
running, and generated segments can be served before the transcode finishes.

That is the correct media-server shape, but it makes resource ownership more
important. A single host can now carry several long-running transcodes, remote
staging jobs, segment reads, heartbeat writes, cleanup scans, and renderer
transports at the same time. Without a single playback admission boundary,
resource policy remains scattered across runner semaphores, config fields,
route helpers, runtime supervisor calls, and diagnostics.

## Relevant Authority

- ADRs:
  - `docs/adr/0005-bounded-async-pipelines-and-resource-budgets.md`
  - `docs/adr/0053-application-control-plane-boundary.md`
  - `docs/adr/0052-hls-runtime-and-media-engine-boundary.md`
  - `docs/adr/0045-ffmpeg-hardware-pipeline-planner.md`
  - `docs/adr/0046-ffmpeg-probe-inventory.md`
  - `docs/adr/0048-playback-transcode-startup-degradation.md`
- Architecture maps:
  - `docs/architecture/PLAYBACK.md`
  - `docs/architecture/CONTROL_PLANE.md`
  - `docs/architecture/WORKSTREAM_LINKS.md`
- Related workstreams:
  - `docs/workstreams/hls-progressive-runtime-boundary/`
  - `docs/workstreams/source-aware-transcode-runtime/`
  - `docs/workstreams/playback-runtime-boundary-deepening/`
  - `docs/workstreams/admin-playback-runtime-diagnostics/`
  - `docs/workstreams/playback-api-transcode-boundary-cleanup/`
  - `docs/workstreams/playback-planner-transcode-value-vocabulary/`

## Problem

Playback runtime limits currently exist, but they do not form one application
boundary:

- `nako-transcode` has CPU/GPU runtime semaphores, but playback admission does
  not first model the complete workload demand.
- Remux, HLS, remote direct stream, and remote staging limits are configured in
  different places and are surfaced mostly as diagnostics.
- `RuntimeSupervisor` owns spawned task tracking, but playback start paths do
  not acquire a unified admission permit before launching process-backed work.
- Public/API routes can still accept work without a host-level pressure reason
  that tells the caller whether the host is busy, unavailable, or unsupported.
- Future remote transcode workers, LL-HLS, DASH, and DRM would all inherit this
  scattered policy if the single-node resource boundary is not made explicit
  first.

## Target State

When this lane closes:

- Playback start paths describe their resource demand before starting process or
  remote I/O work.
- A host-owned playback admission boundary grants or denies bounded permits for
  CPU transcode, GPU transcode, remux process, remote stream, and remote
  staging, while modeling disk-sensitive HLS artifact activity as a follow-on
  pressure class.
- Permit lifetimes cover the actual runtime work that consumes the resource.
- Reuse paths for already-running sessions do not double-acquire process
  permits but still validate access and route readiness.
- Admin diagnostics can explain configured capacity, current pressure, and the
  reason a playback start was rejected, unavailable, unsupported, or
  not-yet-enforced. Queueing remains a follow-on.
- Existing direct, remux, HLS, browser ticket, and renderer transport contracts
  remain stable.

## Shipped Result

This lane shipped the single-node admission boundary described above.

- `nako-server` owns typed `PlaybackResourceDemand` values and runtime
  admission decisions for direct stream, remux, and HLS playback work.
- HLS and remux start paths acquire host-owned permits before launching
  process-backed runtime work.
- Browser playback preflight starts transfer permit ownership into supervised
  background HLS/remux tasks, so immediate route returns do not release capacity
  early.
- Existing active or completed HLS/remux session reuse does not double-acquire
  process permits.
- Admin playback runtime diagnostics expose redaction-safe `resource_pressure`
  with configured capacity, available permits, in-use permits, resource class,
  and enforcement mode.
- Direct remote stream and remote staging pressure are represented by the
  admission/diagnostics vocabulary; the first slice keeps their enforcement on
  the existing host-owned budgets.
- HLS artifact I/O pressure remains modeled as not-yet-enforced. A follow-on
  should bind segment write/read pressure to disk-sensitive admission if real
  operator evidence shows it is needed.

## In Scope

- Single-node playback runtime admission and resource demand modeling.
- CPU/GPU transcode, remux process, remote stream, remote stage, and HLS
  artifact pressure classes.
- Integration with existing `nako-transcode` runtime budgets and server
  `RuntimeSupervisor`.
- Focused Admin/runtime diagnostics needed to explain admission state.
- Tests proving admission, reuse, cancellation, and HTTP responsiveness under
  pressure.

## Out Of Scope

- Distributed transcode workers or external queues.
- LL-HLS, DASH, CMAF encryption, DRM, and key delivery.
- OS-level cgroups, process priority management, or GPU vendor scheduling.
- Player-side ABR/adaptive policy.
- Rewriting the playback planner or changing Public Client wire contracts.
- Database-backed durable job scheduling beyond what existing playback and
  transcode sessions already persist.

## Starting Assumptions

| Assumption | Confidence | Evidence | Consequence if wrong |
| --- | --- | --- | --- |
| Single-node admission is the right first proof before remote workers. | High | ADR 0005 and HLS closeout follow-ons. | Split remote-worker architecture before implementation. |
| Existing config already exposes enough initial capacity knobs. | Medium | `cpu_concurrency`, `gpu_concurrency`, `remux_concurrency`, remote stream/stage settings. | Add a bounded config task before enforcement. |
| `nako-transcode` should still own low-level FFmpeg runner semaphores. | High | Current runner APIs and ADR 0045. | Move only demand/admission vocabulary into server, not process execution. |
| Admission must not change public HLS/remux route contracts. | High | Browser/renderer ticket and playback route workstreams. | Add a client-contract task before changing transport URLs or error shapes. |
| Queueing can be deferred until rejection and bounded permits are explicit. | Medium | Current product has no durable playback job queue. | Open a queue/admission follow-on if users require waiting instead of rejection. |

## Architecture Direction

The target shape is:

```text
Playback request
  -> Playback planner decision
  -> PlaybackResourceDemand
  -> PlaybackRuntimeAdmission
  -> PlaybackRuntimePermit
  -> RuntimeSupervisor / FFmpeg runner / VFS response
  -> diagnostics + persisted session state
```

`nako-server` should own playback admission because it has the user, policy,
session, storage, and route context. `nako-transcode` should keep owning FFmpeg
command planning, hardware execution facts, and low-level runner limits.

Do not make every playback path manually inspect semaphores. Route handlers and
feature helpers should ask one admission boundary for a decision and carry the
permit until the consuming work is done or intentionally transferred to a
supervised runtime task.

## Closeout Condition

This lane closed after:

- playback resource demand has a typed model and test coverage;
- HLS and remux start paths acquire host-owned permits before process-backed
  work starts;
- reuse/cancel/failure paths release or avoid double-acquiring permits
  deterministically;
- Admin diagnostics explain configured capacity and current pressure;
- focused playback/HLS/runtime gates pass with fresh evidence;
- remote workers, LL-HLS/DASH, DRM, queueing semantics, OS isolation,
  per-device tuning, and HLS artifact I/O enforcement are explicitly deferred
  to follow-on lanes.
