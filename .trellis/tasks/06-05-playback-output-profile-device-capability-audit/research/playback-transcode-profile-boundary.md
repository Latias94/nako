# Playback And Transcode Profile Boundary Research

Date: 2026-06-05

## Summary

The current code mostly honors the boundaries required by ADR 0038, ADR 0044,
ADR 0045, ADR 0049, ADR 0052, and ADR 0053.

`nako-playback` is a pure planning crate. It consumes source facts, media probe
facts, playback target facts, effective policy, storage context, and user
preferences, then produces `PlaybackDecision`, `PlaybackDecisionReport`, and
`TranscodeRequirement`. It does not execute FFmpeg, touch HTTP, persist
sessions, or perform storage I/O.

`nako-transcode` owns transcode capability, profile, pipeline, artifact, and
FFmpeg command planning. It owns `HardwareAccelerationReport`,
`HardwareAccelerationPolicy`, `TranscodePipelinePlanner`, `TranscodeProfile`,
HLS request variant identity, HLS artifact manifests, and typed FFmpeg argv
builders. It does not decide the user-visible Direct Play/Remux/Transcode mode.

`nako-server` is the orchestration and control-plane boundary. It loads
source/probe/storage facts, resolves access and playback policy, maps
Public/Admin DTOs, creates runtime diagnostics, manages resource admission,
stages FFmpeg inputs, persists sessions, starts runtime work, serves artifacts,
and handles ticket/renderer transport.

The important gap is not a missing FFmpeg branch. The next playback wave
requires a stable output/device profile contract before HEVC/AV1 output,
hardware tone mapping, image subtitle burn-in, or TV/mobile/native profiles can
move safely in parallel.

The core design rule is:

- device facts must describe the client/player;
- operator policy must describe what the server operator allows;
- runtime capability reports must describe what this host can execute;
- server orchestration combines those facts, but no layer should impersonate
  another layer.

## Evidence

- `crates/nako-playback/src/lib.rs:139`: `PlaybackPlanningRequest` contains
  source, probe, target, effective policy, and context.
- `crates/nako-playback/src/lib.rs:216`: `TranscodeRequirement` is the
  structured playback-to-transcode requirement.
- `crates/nako-playback/src/lib.rs:270`: `ClientPlaybackCapabilities` is still
  a flat capability set.
- `crates/nako-playback/src/lib.rs:399`: `plan_playback` is the planner entry.
- `crates/nako-playback/src/lib.rs:686`: transcode requirements carry output
  constraints, color pipeline, audio output, subtitle strategy, selected
  streams, and reasons.
- `crates/nako-playback/src/capability.rs:119`: `PlaybackTargetProfile` is the
  internal target profile.
- `crates/nako-playback/src/capability.rs:136`: flat client capabilities are
  mapped into one direct profile, one remux profile, and one HLS H264/AAC
  transcode profile.
- `crates/nako-playback/src/capability.rs:180`: profile identity includes
  capability and preference facts that affect planner output.
- `crates/nako-playback/src/capability.rs:356`: direct-play evaluation is pure
  planner logic.
- `crates/nako-playback/src/capability.rs:414`: remux evaluation is pure
  planner logic.
- `crates/nako-playback/src/capability.rs:470`: transcode support evaluation is
  pure planner logic.
- `crates/nako-playback/src/values.rs:95`: color pipeline requirements are
  typed planner values.
- `crates/nako-playback/src/values.rs:183`: audio output requirements are typed
  planner values.
- `crates/nako-playback/src/values.rs:336`: HLS output requirements are typed
  planner values.
- `crates/nako-playback/src/values.rs:343`: subtitle strategy is a typed
  planner value.
- `crates/nako-transcode/src/hardware.rs:31`: hardware accelerators are
  transcode values.
- `crates/nako-transcode/src/hardware.rs:79`: hardware policy is separate from
  runtime capability facts.
- `crates/nako-transcode/src/hardware.rs:96`: pipeline stages include decode,
  filter, encode, hwaccel, tone-map, subtitle burn-in, and bitstream-filter.
- `crates/nako-transcode/src/hardware.rs:395`: hardware capability records
  include stage facts, encoder discovery, device initialization, and smoke
  probe evidence.
- `crates/nako-transcode/src/hardware.rs:601`: FFmpeg hardware detection builds
  a runtime capability report from probes.
- `crates/nako-transcode/src/pipeline.rs:118`: `TranscodePipelineRequest`
  consumes hardware policy, track selection, output constraints, subtitle
  strategy, color pipeline, audio output, and source facts.
- `crates/nako-transcode/src/pipeline.rs:199`: `HlsRuntimePlanRequest` combines
  source, playback transcode plan, hardware policy, track/output/color/audio/
  subtitle/HLS facts, source facts, probe facts, generation, remote input, and
  playback profile key.
- `crates/nako-transcode/src/pipeline.rs:261`: `TranscodePipelinePlanner`
  generates pipeline plans.
- `crates/nako-transcode/src/pipeline.rs:288`: `TranscodePipelinePlanner`
  generates HLS runtime plans and request identity.
- `crates/nako-transcode/src/profile.rs:125`: HLS output policy recognizes
  H264, HEVC, and AV1.
- `crates/nako-transcode/src/profile.rs:429`: HLS validation only allows
  executable H264/AAC; HEVC/AV1 remain deferred unsupported.
- `crates/nako-transcode/src/ffmpeg/hls/encoders.rs:84`: HLS encoder selection
  returns H264 encoder names only.
- `crates/nako-transcode/src/ffmpeg/hls/filters.rs:14`: software HDR-to-SDR
  tone mapping is currently implemented as a software filter chain.
- `crates/nako-transcode/src/ffmpeg/hls/filters.rs:77`: color filter planning
  rejects deferred HDR and requires software for current tone mapping.
- `crates/nako-transcode/src/ffmpeg/hls/filters.rs:125`: subtitle burn-in is
  filter-graph planning and currently requires the software pipeline.
- `crates/nako-server/src/app/playback/support.rs:52`: support evidence loads
  redaction-safe source/session context.
- `crates/nako-server/src/http/admin.rs:2904`: Admin runtime diagnostics adapt
  playback runtime state into wire DTOs.
- `docs/architecture/PLAYBACK.md`: Lane A identifies Device Capability
  Profiles as the next capability profile lane.
- `docs/adr/0038-playback-planning-and-transcode-policy-seams.md`: playback
  planning, transcode policy, runtime inventory, Admin settings, and Public
  DTOs are separate seams.
- `docs/adr/0044-playback-capability-profile-planner.md`: `nako-playback`
  owns pure planner/profile records, while Public Client DTOs map into that
  model.
- `docs/adr/0045-ffmpeg-hardware-pipeline-planner.md`: `nako-transcode` owns
  stage-aware hardware capability and command planning.
- `docs/adr/0049-source-aware-transcode-runtime.md`: source facts flow through
  playback requirements into transcode pipeline planning.
- `docs/adr/0052-hls-runtime-and-media-engine-boundary.md`: Nako owns planning,
  identity, runtime, and safe serving; FFmpeg/ffprobe own media execution.
- `docs/adr/0053-application-control-plane-boundary.md`: policy, diagnostics,
  resource accounting, and redaction are control-plane concerns.

## Current Capabilities

### `nako-playback`

The planner can currently express:

- Direct Play, Remux, Transcode, and Denied decisions;
- separate direct/remux/transcode capability evaluations;
- typed compatibility reasons;
- remote and range-readable storage facts;
- requested audio/subtitle streams;
- preferred audio/subtitle languages;
- max video bitrate and HDR preference;
- requested remux or transcode output container;
- selected stream facts including codec, profile, level, pixel format, bit
  depth, HDR/color facts, audio channels, and subtitle flags;
- audio output requirements for downmix and normalization;
- color pipeline requirements for HDR-to-SDR or deferred HDR;
- HLS output variant and segment container;
- subtitle delivery strategy: none, preserve, omit, sidecar, or burn-in.

This is enough for the current shipped planner, but not enough for a durable
device profile database or conditional output matrix.

### `nako-transcode`

The transcode boundary can currently express:

- CPU, VAAPI, NVENC, Quick Sync, AMF, and VideoToolbox acceleration;
- requested accelerator and CPU/fail fallback policy;
- hardware capability reports with stage-level facts;
- source-aware pipeline readiness with requested/selected/fallback evidence;
- transcode profile identity;
- HLS request variant identity;
- adaptive ladder and media rendition identity;
- artifact manifests and artifact allow-lists;
- FFmpeg input, device, filter, encoder, muxer, sidecar, and seek command
  planning.

The executable HLS output remains intentionally narrow. HEVC/H265 and AV1 are
recognized as policy values, but validation rejects them as deferred unsupported
and FFmpeg HLS encoder selection still emits H264 encoders. Hardware inventory
can record optional HEVC/AV1 and tone-map capabilities, but the executable path
does not use them for output codec selection.

### `nako-server`

The server currently:

- maps Public playback and renderer DTOs into `ClientPlaybackCapabilities` and
  `PlaybackTarget`;
- loads source, probe, storage, access, and effective playback policy facts;
- calls `PlaybackPlanner`;
- maps playback transcode requirements into transcode values;
- calls `TranscodePipelinePlanner` for HLS;
- manages remux and renderer orchestration through focused flow modules;
- stores hardware reports and HLS readiness in the HLS app service;
- handles resource admission for immediate, HLS start, and HLS supersede
  policies;
- adapts playback runtime state into Admin diagnostics and support evidence.

## Gaps

### 1. Device And Output Profile Inputs Are Too Shallow

`ClientPlaybackCapabilities` and `ClientPlaybackCapabilitiesDto` remain flat.
They cannot represent codec/container condition rows, codec profile or level,
bit depth, frame rate, HDR format matrices, subtitle delivery matrices, audio
passthrough/downmix behavior, HLS output codec, LL-HLS/CMAF/DASH support,
device family, profile version, or player engine.

### 2. `PlaybackTargetProfile` Is Profile-Shaped But Not A Durable Contract

The internal profile has direct/remux/transcode profile vectors, but
`from_capabilities` currently builds a fixed legacy shape:

- one direct-play profile from flat client fields;
- one remux profile with MP4/MKV output containers;
- one transcode profile with HLS H264/AAC output.

That is a good adapter for current behavior, but it is not yet a versioned
device/output profile contract.

### 3. Transcode Pipeline Planning Does Not Yet Select By Output Codec

`HlsRuntimePlanRequest` contains the `TranscodePlan`, but
`TranscodePipelineRequest` does not directly carry target output video/audio
codec into stage selection. H264 baseline is safe today. HEVC/AV1 execution
will require target output codec to participate in pipeline readiness, encoder
stage selection, fallback, profile identity, and command planning before the
FFmpeg builder runs.

### 4. Runtime Hardware Facts Are Wider Than Executable Output

`hardware.rs` records optional HEVC/AV1 decoder/encoder, tone-map filters,
subtitle burn-in filters, device initialization, and smoke probes. That is
runtime evidence, not permission to expose HEVC/AV1 or hardware tone mapping as
executable output. The executable policy must remain explicit.

### 5. Operator Policy Is Too Global For The Next Wave

Current transcode config has global hardware acceleration, fallback, and CPU/GPU
concurrency. Future work likely needs explicit operator policy for:

- output codec enablement;
- per-accelerator and per-stage fallback;
- experimental HEVC/AV1 gates;
- hardware smoke requirements;
- quality/bitrate defaults;
- resource-budget interaction.

These are operator/runtime policy facts and must not be encoded as client
capability facts.

### 6. Admin Needs Effective Profile Support Evidence

Admin runtime diagnostics already expose hardware, pipeline, budget, resource,
staging, artifact, and throttle facts. Public decision responses expose direct/
remux/transcode evaluations. Admin support evidence still lacks a safe summary
of the effective profile, selected output, and decision reason matrix for a
specific support case.

## Recommended Contract Layers

### Public Client And Device Output Profile Facts

Owner: `nako-client-protocol`, `nako-api`, server HTTP mapping, and
`nako-playback` domain adapters.

These facts should describe the client/player, not the server host:

- `profile_id`, `profile_version`, `device_family`, `player_engine`;
- direct-play profile rows;
- remux profile rows;
- transcode output profile rows;
- subtitle delivery profile rows;
- audio output facts;
- color pipeline facts;
- HLS output facts;
- legacy flat fields mapped into a `legacy_default` profile row.

Forbidden here: FFmpeg path, encoder names, GPU device path, server hardware
availability, operator fallback policy, and resource pressure.

### Playback Planner Contract

Owner: `nako-playback`.

The planner chooses Direct Play, Remux, Transcode, or Denied from source facts,
media technical facts, client profile facts, effective playback policy, storage
facts, and preferences. It outputs `PlaybackDecisionReport` and
`TranscodeRequirement`. It says what output is needed and why; it does not say
which GPU or command line executes it.

### Transcode Pipeline Capability And Execution Contract

Owner: `nako-transcode`.

Transcode consumes playback requirements, operator hardware policy, runtime
hardware reports, and source facts. It selects decode/filter/encode/tone-map/
subtitle stages, fallback, readiness, profile identity, artifact identity, and
FFmpeg command plans. Command builders consume typed plans and must not re-run
playback policy.

### Operator And Runtime Policy

Owner: server config, Admin API, and the control plane.

Operator policy owns hardware acceleration, fallback, resource budgets, output
codec gates, per-stage policy, runtime supervision, admission, cleanup, and
throttle. It can disallow or constrain an executable path, but it must not claim
that a client supports a capability.

### Admin Diagnostics And Support Evidence

Owner: `nako-api/src/admin/playback.rs`, `nako-server/src/http/admin.rs`, and
Admin Web.

Admin should expose redaction-safe hardware, pipeline, fallback, budget,
resource, profile-summary, selected-output, and reason-matrix facts. It must
not expose raw source locators, local paths, cache URIs, FFmpeg commands, raw
stderr, tokens, secrets, or device paths.

## Recommended Follow-Ons

### First Executable Follow-On

`playback-output-profile-v2-skeleton-contract-only`

Scope:

- add only additive optional contract skeleton fields;
- keep existing flat fields and playback behavior unchanged;
- map flat capabilities into a `legacy_default` profile row;
- add identity and mapping tests that prove absent v2 fields do not change
  decisions;
- do not enable HEVC/AV1, hardware tone mapping, or image subtitle burn-in.

Precondition or companion gate:

- `public-client-playback-capability-contract-parity-gate`, to align current
  capability fields across protocol, OpenAPI, Rust client, client core, SDKs,
  and HTTP docs.

### Ranked Follow-Ons

1. `public-client-playback-capability-contract-parity-gate`
   - Fix current field coverage drift without behavior changes.

2. `playback-output-profile-v2-skeleton-contract-only`
   - Establish device family/profile rows/legacy mapping without enabling new
     execution.

3. `browser-mobile-tv-renderer-profile-fixtures`
   - Add fixture profiles and planner matrix tests after the skeleton lands.

4. `hls-hevc-av1-output-policy-design-and-first-exec`
   - Requires client output profile facts, operator codec gates, output codec
     in pipeline selection, and profile identity decisions.

5. `hardware-tone-map-execution-first-slice`
   - Requires hardware filter/tone-map stage selection, device init/smoke
     policy, and fallback policy. It must not run in parallel with HEVC/AV1
     execution.

6. `image-subtitle-burn-in-capability-and-execution`
   - Requires subtitle delivery profiles for image/text, embedded/external,
     sidecar, and burn-in requirements.

7. `admin-playback-effective-profile-support-evidence`
   - Can follow the profile skeleton as an Admin-only support evidence slice.

## Unsafe Parallel Work

- HEVC/AV1 output execution in parallel with hardware tone-map execution.
- Multiple tasks editing `PlaybackTargetProfile::identity`,
  `ClientPlaybackCapabilitiesDto`, or generated Public Client contracts.
- Multiple tasks editing `TranscodeProfile` identity, `HlsRequestVariantPlan`,
  or `HlsArtifactManifest` reconstruction.
- Admin support evidence expansion in parallel with Public Client profile DTO
  expansion without a contract owner.
- HLS lifecycle/admission work in parallel with VFS/remote staging or
  circuit-breaker changes in the same playback flow files.

## Changes Made By This Research

This research changed only this task's research documentation. It did not
modify production Rust, TypeScript, DTOs, routes, OpenAPI generation, SDK
generation, architecture maps, or tests.
