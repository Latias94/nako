# Playback, Decoding, And Transcode Lane Audit

Date: 2026-06-05

## Scope

This note audits the `playback-transcode` lane for the cross-lane architecture
audit task. It is based on:

- `CONTEXT.md`
- `docs/architecture/PLAYBACK.md`
- `docs/architecture/LANES.md`
- ADR 0038, 0044, 0045, 0049, 0052, and 0053
- `crates/nako-playback`
- `crates/nako-transcode`
- `crates/nako-server/src/app/playback`
- related Admin playback DTOs, route mappings, specs, and system tests

No production code was changed.

## 1. Current Architecture State

### Overall State

The playback/transcode lane is not in a shallow-module emergency. The main ADR
split is mostly honored:

- `nako-playback` owns the pure Playback Runtime planning model: source facts,
  target capabilities, effective policy, selected mode, decision report,
  track selection, subtitle strategy, audio output requirements, and color
  pipeline requirements.
- `nako-transcode` owns Transcode Pipeline planning, FFmpeg command planning,
  hardware inventory, HLS artifact manifests, profile/request identity,
  remux/HLS execution requests, and FFmpeg runner adapters.
- `nako-server/src/app/playback` owns orchestration: source/probe loading,
  access and policy checks, VFS input staging, playback session persistence,
  transcode session reuse, resource admission, HLS supersede/cancellation,
  artifact serving, and Admin support evidence.
- `nako-api/src/admin/playback.rs` keeps Admin DTOs redaction-safe and separate
  from pure planner records.

The shipped surface is broad for the current Video-First Phase:

- Direct Play byte/range serving exists.
- Remux lifecycle has its own server-side flow module.
- HLS runtime exists with progressive output visibility, fMP4, adaptive ladder,
  audio sidecars, subtitle sidecars, text subtitle burn-in planning/execution,
  software HDR-to-SDR tone mapping, seek generation identity, and bounded
  startup admission.
- Runtime diagnostics expose FFmpeg/hardware readiness, resource pressure,
  staging/artifact lifecycle, playback sessions, and redaction-safe support
  evidence.

### Important Evidence

- `crates/nako-playback/src/lib.rs` has one pure `PlaybackPlanner` returning
  `PlaybackDecision` with `PlaybackDecisionReport` and a structured
  `TranscodeRequirement`.
- `crates/nako-playback/src/capability.rs` evaluates Direct Play, Remux, and
  Transcode through capability profiles and typed compatibility conditions.
- `crates/nako-playback/src/values.rs` carries domain vocabulary for audio
  output, HDR tone mapping, HLS output shape, and subtitle strategy.
- `crates/nako-transcode/src/pipeline.rs` maps Playback Transcode requirements
  and Hardware Capability Reports into `HlsRuntimePlan`,
  `TranscodePipelinePlan`, execution policy, request identity, media rendition
  identity, and source-aware hardware fallback.
- `crates/nako-transcode/src/ffmpeg/hls/*` is split by command concern:
  input, filters, encoders, muxer, seek, and sidecars. This is a healthy
  implementation shape for narrow execution follow-ons.
- `crates/nako-server/src/app/playback/hls_flow.rs`,
  `remux_flow.rs`, and `renderer_flow.rs` keep the app root thin for the
  riskiest playback workflows.
- `.trellis/spec/nako-server/backend/directory-structure.md` already records
  the expected HLS, Remux, renderer, staging, and resource-admission shapes.

### Gaps And Friction

1. **Device/profile breadth is still the main planning gap.**
   `ClientPlaybackCapabilities` is deeper than the original codec-list shape,
   but it is still not a full device profile system. It does not yet represent
   enough per-device subtitle, HDR, codec profile/level, bitrate, audio output,
   or output-codec preference to safely expand HEVC/AV1, hardware tone mapping,
   TV profiles, DLNA, AirPlay, and mobile/native playback in parallel.

2. **HEVC/AV1 output is intentionally deferred.**
   `nako-transcode/src/profile.rs` recognizes HEVC/AV1 HLS output policy but
   rejects it as not executable. Hardware inventory already probes optional
   HEVC/AV1 encoders/decoders, but command execution remains H264/AAC.

3. **Hardware tone mapping needs a design slice before implementation.**
   Software HDR-to-SDR exists. Hardware tone mapping is not just another
   encoder switch; it affects decode/filter/upload/download/device setup and
   FFmpeg filter graph shape per VAAPI/CUDA/QSV/etc.

4. **Image subtitles and external subtitle burn-in are intentionally blocked.**
   Text subtitle burn-in is executable on the software pipeline. PGS/image
   subtitle burn-in, external subtitle burn-in, hardware-filter burn-in, and
   richer client subtitle profiles remain follow-ons.

5. **Resource admission is process-local, not a durable queue.**
   HLS startup now has bounded wait policies (`HlsStart`, `HlsSupersede`), but
   durable queueing, remote workers, per-artifact read/write pressure, and
   control-plane job scheduling are still separate ADR 0053 work.

6. **Architecture map drift exists.**
   `docs/architecture/PLAYBACK.md` still describes public
   `start_position_ms` playlist query as a follow-on, while code already has
   `start_position_ms`, `HlsPlaybackGeneration`, request-variant identity, and
   FFmpeg seek args. The remaining seek work is keyframe/timestamp/player
   validation, not initial query plumbing.

## 2. Candidate Next Tasks

### 1. `playback-output-profile-and-device-capability-audit`

Classification: needs architecture audit.

Recommended priority: P0.

Scope:

- `docs/architecture/PLAYBACK.md`
- ADR 0044 / ADR 0045 follow-on notes
- `crates/nako-playback`
- `crates/nako-transcode/src/profile.rs`
- Public Client and Admin playback DTO boundaries
- Web/native/renderer capability assumptions

Goal:

Decide the next stable capability profile shape before adding more execution
features. The audit should answer:

- How should Client Applications report output codec/container/subtitle/HDR
  support beyond the current default profile?
- Which facts belong in Public Client API versus Admin diagnostics?
- Should HEVC/AV1 output be exposed as explicit requested output, device
  profile default, or hidden operator-only experiment first?
- What is the minimum device profile model for browser, mobile, TV, renderer,
  and desktop/native playback?

Why this comes first:

Most next execution features depend on this decision. Without it, HEVC/AV1,
hardware tone mapping, subtitle burn-in, and player UX can each invent their
own capability inputs.

Parallel safety:

Not safe to parallelize with any implementation touching `nako-playback`
capability structs, Public Client playback DTOs, or generated client contracts.

### 2. `playback-architecture-map-seek-status-reconciliation`

Classification: ready bounded implementation; architecture-map/doc
reconciliation.

Recommended priority: P1, but can run in parallel with code-free lane audits.

Scope:

- `docs/architecture/PLAYBACK.md`
- `docs/architecture/LANES.md`
- possibly `.trellis/spec/nako-server/backend/directory-structure.md` if a
  stale seek note exists

Goal:

Reconcile the map with current code:

- `start_position_ms` is already accepted by the HTTP HLS route.
- `HlsPlaybackGeneration` already participates in HLS request-variant identity.
- FFmpeg seek command args already exist.
- Remaining seek work is keyframe/timestamp discipline, player validation,
  segment-window behavior, and restart UX.

Why this is bounded:

No production code is required. The task only prevents future planners from
opening an already-shipped first slice.

Parallel safety:

Safe if no other lane is editing the same architecture maps.

### 3. `admin-playback-support-source-facts-first-slice`

Classification: ready bounded implementation.

Recommended priority: P2.

Scope:

- `crates/nako-api/src/admin/playback.rs`
- `crates/nako-server/src/http/admin.rs`
- `crates/nako-server/src/app/playback/support.rs`
- focused server system tests
- optional Admin Web rendering only if the broader Admin/API audit selects it

Goal:

Extend Admin playback support evidence with redaction-safe media fact summary:

- has media probe
- video/audio/subtitle stream counts
- selected source scheme
- selected output artifact kind
- selected acceleration/fallback already present
- no raw locator, path, command line, stderr, request key, or token leakage

Why it is useful:

Operators debugging Playback Transcode failures need to know whether Nako had
enough Media Technical Facts to explain a decision. Existing support evidence
is safe but intentionally sparse.

Why it is bounded:

The existing Admin support evidence route, DTO pattern, and redaction tests are
already in place. The task can stay Admin-only and avoid Public Client API
changes.

Parallel safety:

Serializes with any Admin/API generated-contract task, Admin Web playback page
work, or playback support evidence redaction work.

### 4. `hls-hevc-av1-executable-output-first-slice`

Classification: needs architecture audit before implementation.

Recommended priority: P2 after candidate 1.

Scope:

- `crates/nako-transcode/src/profile.rs`
- `crates/nako-transcode/src/pipeline.rs`
- `crates/nako-transcode/src/ffmpeg/hls/encoders.rs`
- `crates/nako-transcode/src/hardware.rs`
- `crates/nako-playback` output profile inputs
- server HLS tests if exposed through runtime flow

Goal:

Turn deferred HEVC/AV1 output policy into a narrow executable slice.

Possible first executable shape:

- software HEVC only, fMP4 only, operator-disabled by default; or
- hardware-specific HEVC only for one accelerator; or
- command-plan-only support with Admin diagnostics but no route exposure.

Audit question:

This cannot start safely until the lane decides how clients request or receive
HEVC/AV1 output and how fallback works when a target profile cannot play it.

Parallel safety:

Not safe with hardware tone mapping, adaptive ladder changes, or any task
changing `TranscodeProfile` identity.

### 5. `hardware-tone-map-execution-first-slice`

Classification: needs architecture audit before implementation.

Recommended priority: P2/P3 after candidate 1.

Scope:

- `crates/nako-transcode/src/pipeline.rs`
- `crates/nako-transcode/src/hardware.rs`
- `crates/nako-transcode/src/ffmpeg/hls/filters.rs`
- `crates/nako-transcode/src/ffmpeg/hls/input.rs`
- operations/release hardware smoke evidence

Goal:

Choose one hardware tone-map path and make it executable without weakening the
existing software HDR-to-SDR fallback.

Why this needs audit:

Hardware tone mapping changes decode/filter stages, device initialization,
filter graph layout, and fallback semantics. VAAPI, CUDA/OpenCL, QSV, AMF, and
VideoToolbox do not share one safe FFmpeg command shape.

Parallel safety:

Not safe with HEVC/AV1 output execution or hardware report schema changes
unless one planner owns the shared `Hardware Capability Report` contract.

### 6. `playback-artifact-identity-and-flow-cleanup`

Classification: fearless refactor cleanup candidate.

Recommended priority: P3; do not run before the P0 audit unless the chosen
implementation task touches these files anyway.

Scope:

- `crates/nako-transcode/src/artifact.rs`
- `crates/nako-transcode/src/profile.rs`
- `crates/nako-server/src/app/playback/mod.rs`
- `crates/nako-server/src/app/playback/selection.rs`

Problem:

The core modules are directionally deep, but several files are now large enough
to slow future agents:

- `artifact.rs` owns HLS rendition records, artifact manifests, request variant
  identity, identity parsing, cleanup decisions, and many test fixtures.
- `profile.rs` owns output profile validation, deferred HEVC/AV1 policy, and
  identity building.
- `app/playback/mod.rs` still contains the broad service type, trait adapter,
  request/response records, and thin entry points even though lifecycle logic
  has moved into flow modules.

Good cleanup shape:

- Split HLS identity parsing from artifact manifest serving logic.
- Move shared test builders into local test helper modules without changing
  public behavior.
- Extract app playback request/response records from the broad root only if it
  reduces conflict and improves locality.

Why not first:

This is cleanup, not an urgent correctness fix. Running it before the output
profile audit could create churn in the exact files future implementation lanes
need to touch.

Parallel safety:

Unsafe with almost all playback implementation work. Run alone or piggyback on
the selected feature lane.

## 3. Classification Summary

Ready bounded implementation:

- `playback-architecture-map-seek-status-reconciliation`
- `admin-playback-support-source-facts-first-slice`

Needs architecture audit:

- `playback-output-profile-and-device-capability-audit`
- `hls-hevc-av1-executable-output-first-slice`
- `hardware-tone-map-execution-first-slice`

Fearless refactor cleanup candidate:

- `playback-artifact-identity-and-flow-cleanup`

Needs more product decision before implementation:

- Full device profile database for TV/mobile/native/renderer clients
- HEVC/AV1 output exposure and fallback policy
- Hardware tone mapping accelerator order
- Image subtitle and external subtitle behavior
- Durable playback queueing and remote transcode workers

Unsafe to parallelize without planner coordination:

- Any two tasks changing `PlaybackTargetProfile`, `ClientPlaybackCapabilities`,
  or Public Client playback DTOs.
- Any two tasks changing `TranscodeProfile` identity, HLS request identity, or
  `HlsArtifactSpec` reconstruction.
- Hardware tone-map execution and HEVC/AV1 output execution.
- HLS lifecycle/admission work and storage/VFS staging/circuit-breaker work.
- Admin support evidence and broader Admin/API/generated contract work.

## 4. Parallel Development Conflict Surface

### High-Conflict Files

- `crates/nako-playback/src/lib.rs`
- `crates/nako-playback/src/capability.rs`
- `crates/nako-playback/src/values.rs`
- `crates/nako-transcode/src/pipeline.rs`
- `crates/nako-transcode/src/profile.rs`
- `crates/nako-transcode/src/artifact.rs`
- `crates/nako-transcode/src/hardware.rs`
- `crates/nako-transcode/src/ffmpeg/hls/*`
- `crates/nako-server/src/app/playback/hls_flow.rs`
- `crates/nako-server/src/app/playback/hls.rs`
- `crates/nako-server/src/app/playback/resource.rs`
- `crates/nako-server/src/app/playback/selection.rs`
- `crates/nako-api/src/admin/playback.rs`
- `crates/nako-server/src/http/admin.rs`
- `crates/nako-server/src/http/playback.rs`

### Cross-Lane Shared Scopes

- Storage/VFS: FFmpeg input staging, remote source reads, staging lease cleanup,
  remote stream/stage permits, future Storage Circuit Breaker behavior.
- Control plane: durable queueing, resource classes, runtime supervision,
  diagnostics, trace context, and ADR 0053 job policy.
- Admin/API/Web: playback runtime diagnostics, support evidence, playback
  session lists, generated contract updates, Admin Web route state.
- Operations/release: FFmpeg presence, hardware report evidence, container
  device pass-through, one-frame GPU smoke, release gates.
- Client surfaces: browser player capability reporting, renderer transport
  tickets, mobile/native profiles, future desktop/native playback.

### Safe Parallel Shapes

These can run in parallel if each task owns its scope:

- Playback output/profile audit as a docs/research task.
- Remote access/network tunnel audit in the operations/control-plane lane.
- Addon boundary audit focused on Playback Resource Suggestions, not Playback
  Runtime execution.
- Storage/VFS source identity audit, as long as it does not change playback
  input staging.
- Admin/API audit, as long as it does not edit playback DTOs at the same time
  as `admin-playback-support-source-facts-first-slice`.

## 5. Recommended Priority

Recommended plan: mixed plan, but audit first.

1. Run `playback-output-profile-and-device-capability-audit`.
   This is the highest-value next step because it prevents HEVC/AV1, hardware
   tone mapping, subtitle execution, and player UX lanes from creating
   incompatible capability inputs.

2. In parallel, run `playback-architecture-map-seek-status-reconciliation`.
   This is low-risk and prevents stale architecture guidance from generating
   duplicate seek work.

3. If an Admin/API lane is available, run
   `admin-playback-support-source-facts-first-slice` after confirming it does
   not collide with broader Admin contract work.

4. Do not start `hls-hevc-av1-executable-output-first-slice` or
   `hardware-tone-map-execution-first-slice` until the output/profile audit
   chooses the capability contract and first executable target.

5. Defer `playback-artifact-identity-and-flow-cleanup` until after the next
   implementation target is chosen. It is a valid fearless refactor candidate,
   but not the next best first move.

Bottom line:

The playback lane should not jump directly into broad fearless refactor cleanup.
The current architecture is deep enough to support narrow follow-ons. The next
planning bottleneck is profile/output capability architecture, followed by one
or two bounded implementation slices.
