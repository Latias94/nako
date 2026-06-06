# Quality Guidelines

Transcode changes must keep FFmpeg behavior typed, bounded, and testable.

## Required Patterns

- Build FFmpeg commands through typed request and command builder structs.
- Assert exact argv shape in unit tests for new command-planning behavior.
- Use artifact manifests before publishing HLS/remux outputs to callers.
- Keep overwrite policy explicit.
- Keep hardware acceleration policy and capability reports explicit; do not
  probe GPU support for every playback request.
- Keep HLS output codec policy typed before enabling new FFmpeg encoder paths.
  H264 is the executable baseline; HEVC/H265 and AV1 are recognized future
  policy values until a dedicated execution slice wires encoder args and
  compatibility gates.
- Keep runtime limits bounded with concurrency and timeout values.

## Forbidden Patterns

- Do not concatenate FFmpeg command strings by hand.
- Do not publish HLS playlists or segment paths unless a typed manifest proves
  they are generated and safe to expose.
- Do not default to transcode when Direct Play or Remux is compatible; that is
  a playback planner decision.
- Do not assume HLS segment container, codec, or hardware acceleration support
  without a policy/capability fact.
- Do not hide CPU/GPU/disk/network pressure inside the FFmpeg builder.

## Tests Required

- Unit tests for command argv, overwrite policy, in-place output rejection, and
  HLS artifact manifest shape.
- Unit tests for hardware capability and degraded inventory behavior.
- Unit tests for readiness/fallback coupling when hardware selection can
  degrade to CPU.
- Runtime tests for concurrency limits, timeout, progress, and cancellation when
  those behaviors change.
- Server integration tests when admission, playback route, or artifact serving
  behavior changes.
- Process-backed HLS runner tests that assert serve-while-running artifacts
  must wait with a bounded polling helper rather than immediately checking the
  filesystem. Full workspace nextest can schedule many fake FFmpeg processes at
  once, especially on Windows, so artifact-readiness waits should cover normal
  process startup jitter without hiding a real hang.

## Scenario: HLS FFmpeg Command Part Locality

### 1. Scope / Trigger

- Trigger: changing `crates/nako-transcode/src/ffmpeg/hls.rs` command-part
  assembly, including input args, primary HLS output args, sidecar outputs,
  encoder args, muxer args, or filter args.

### 2. Signatures

- Public builder shape remains
  `FfmpegCommandBuilder::hls(&HlsRequest) -> Result<FfmpegCommandPlan>`.
- FFmpeg argv remains `Vec<FfmpegArg>`; do not add stringly command fragments.

### 3. Contracts

- Input/global args are emitted first, then the primary HLS output, then
  sidecar outputs.
- Single-variant and adaptive HLS should share request-derived output facts
  such as `main_output_has_audio` and the planned audio filter graph.
- Seek/restart FFmpeg args for a given HLS request should derive from a single
  typed seek command plan, not from repeated `HlsPlaybackGeneration` checks in
  input, encoder, and muxer builders.
- Single-variant and adaptive HLS should branch only where their primary output
  command differs: stream maps, encoder args, and muxer args.
- Playback planning, server HLS lifecycle, and artifact publication authority
  must stay outside the FFmpeg command builder.

### 4. Validation & Error Matrix

| Condition | Behavior |
|-----------|----------|
| Empty HLS input path | invalid input planning error |
| Input path equals primary playlist path | reject command plan |
| Subtitle artifacts without sidecar-selected strategy | invalid input planning error |
| Sidecar-selected strategy without subtitle artifacts | invalid input planning error |
| Burn-in-selected strategy without selected subtitle stream | invalid input planning error |
| Burn-in-selected strategy without probe-confirmed text subtitle codec facts | invalid input or unsupported planning error |
| Burn-in-selected strategy with external or image subtitle facts | unsupported planning error |
| Burn-in-selected strategy with subtitle sidecar artifacts | invalid input planning error |
| Burn-in-selected strategy on hardware filter pipeline | unsupported FFmpeg adapter error |
| Burn-in-selected strategy with selected subtitle and software pipeline | primary `-vf subtitles=...:si=<subtitle-ordinal>` filter, no sidecar output |
| Preserve-in-container subtitles | unsupported FFmpeg adapter error |

### 5. Good / Base / Bad Cases

- Good: group command args by input, primary output, and sidecar outputs; derive
  shared output facts once and pass them into typed part builders.
- Base: single-variant HLS and adaptive HLS each provide their own primary
  output stream-map, encoder, and muxer args.
- Bad: recompute audio-filter or main-output-audio decisions separately in
  single/adaptive branches or splice sidecar args into the primary output block.

### 6. Tests Required

- Exact argv test when changing HLS command-part ordering.
- Include at least one path with primary output plus audio/subtitle sidecar
  outputs when touching sidecar locality.
- Include exact argv coverage for subtitle burn-in filters when changing
  burn-in or video filter graph planning. The FFmpeg `si` value is an ordinal
  among subtitle streams, not Nako's global source stream index.
- Include exact argv coverage for default and non-default HLS seek generations
  when changing seek input, keyframe, timestamp, or HLS flag planning.
- Run `cargo nextest run -p nako-transcode hls --no-fail-fast` for HLS command
  part changes.

### 7. Wrong vs Correct

#### Wrong

```rust
let audio_filter_graph = filters::hls_audio_filter_graph(policy.audio_output)?;
let single = sidecars::hls_audio_sidecar_args(artifacts, segment_time, audio_filter_graph.as_deref());
let audio_filter_graph = filters::hls_audio_filter_graph(policy.audio_output)?;
let adaptive = sidecars::hls_audio_sidecar_args(artifacts, segment_time, audio_filter_graph.as_deref());
```

#### Correct

```rust
let output = HlsOutputAssemblyContext::from_request(request)?;
let primary_output = single_variant_primary_output_parts(request, &output)?;
let sidecar_outputs = FfmpegHlsSidecarOutputParts::from_request(request, &output);
```

## Scenario: Pipeline Readiness And Fallback Coupling

### 1. Scope / Trigger

- Trigger: changing `TranscodePipelineReadiness`,
  `TranscodeAccelerationFallbackPlan`, hardware pipeline selection, source-aware
  decode compatibility, HDR tone-mapping fallback, or unavailable-pipeline
  diagnostics.

### 2. Signatures

- `TranscodePipelinePlanner::plan_hls_single_variant(...) ->
  Result<TranscodePipelinePlan>` returns both `readiness` and
  `acceleration.fallback`.
- `TranscodePipelineReadiness` is the selected-pipeline authority for
  `requested`, `selected`, and `fallback_used`.
- `TranscodeAccelerationFallbackPlan` must be derived from the selected
  readiness plus the requested fallback policy, not rebuilt from parallel local
  variables.

### 3. Contracts

- CPU-requested readiness must report `requested = None`, `selected = None`,
  `fallback_used = false`, and reason `CpuRequested`.
- Requested-pipeline-ready readiness must report requested and selected as the
  requested accelerator.
- Degraded CPU fallback readiness must report requested as the hardware
  accelerator, selected as `None`, and `fallback_used = true`.
- Unavailable readiness must preserve the requested accelerator and an explicit
  unavailable reason; it must not claim fallback was used.
- Any `TranscodePipelinePlan.acceleration.fallback` must mirror the plan's
  `readiness.requested`, `readiness.selected`, and `readiness.fallback_used`.

### 4. Validation & Error Matrix

| Condition | Behavior |
|-----------|----------|
| CPU requested and software available | Ready / `CpuRequested` |
| Requested hardware available and source compatible | Ready / `RequestedPipelineReady` |
| Requested hardware unavailable and CPU fallback available | Degraded / CPU selected |
| Requested hardware source decode incompatible and CPU fallback available | Degraded / CPU selected with source incompatibility reason |
| CPU fallback unavailable | `NakoError::Unsupported` or unavailable readiness for diagnostics |
| HDR tone mapping requires software fallback | Degraded to CPU when fallback policy allows it |

### 5. Good / Base / Bad Cases

- Good: construct readiness through helper constructors and derive fallback from
  readiness.
- Base: a ready hardware plan and its fallback plan both name the same hardware
  accelerator as requested and selected.
- Bad: manually copy `policy.requested`, `selection.selected`, and
  `selection.fallback_used` into a fallback plan in a separate block; future
  readiness changes can drift.

### 6. Tests Required

- Test degraded source-aware hardware decode fallback and assert fallback plan
  fields mirror readiness fields.
- Test HDR tone-mapping fallback and CPU-requested paths when those paths
  change.
- Run `cargo nextest run -p nako-transcode hls_pipeline --no-fail-fast` for
  focused HLS pipeline changes.

### 7. Wrong vs Correct

#### Wrong

```rust
let fallback = TranscodeAccelerationFallbackPlan {
    requested: policy.requested,
    selected: selection.selected,
    fallback: policy.fallback,
    fallback_used: selection.fallback_used,
};
```

This lets readiness and fallback drift if selected-pipeline semantics change.

#### Correct

```rust
let fallback = selection.acceleration_fallback_plan(policy.fallback);
```

The selected readiness remains the single source of truth for the fallback
summary embedded in the execution plan.

## Scenario: HLS Output Codec Policy

### 1. Scope / Trigger

- Trigger: changing HLS transcode profile video codec validation or preparing
  future HEVC/AV1 HLS output behavior.

### 2. Signatures

- `HlsVideoOutputPolicyDecision::from_requested_codec(Option<&str>) ->
  HlsVideoOutputPolicyDecision`.
- `TranscodeProfile::validate_hls(...)` consumes the decision before profile
  identity or FFmpeg command planning is allowed.

### 3. Contracts

- Omitted HLS video codec means the H264 baseline.
- Explicit H264/AVC is executable.
- Explicit HEVC/H265 and AV1 are recognized but deferred unsupported.
- Unknown HLS video codecs are unsupported.
- AAC remains the only executable HLS audio codec in this slice.
- Recognizing HEVC/AV1 must not change playback defaults, public API DTOs,
  server routes, HLS artifacts, or FFmpeg encoder argv.

### 4. Validation & Error Matrix

| Condition | Behavior |
|-----------|----------|
| HLS video codec omitted | valid H264 baseline |
| HLS video codec H264/AVC | valid executable profile |
| HLS video codec HEVC/H265 | validation error: deferred unsupported |
| HLS video codec AV1 | validation error: deferred unsupported |
| HLS video codec unknown | validation error: unsupported |
| HLS audio codec not AAC | validation error: audio unsupported |

### 5. Good / Base / Bad Cases

- Good: add typed policy vocabulary first, then future execution slices can
  choose encoder args with explicit compatibility gates.
- Base: HLS profile validation accepts H264/AAC and exact HLS FFmpeg argv tests
  continue to prove H264 encoder behavior.
- Bad: add `hevc_*` or `av1_*` encoder names to the FFmpeg HLS builder without
  profile validation, client compatibility, segment/container, and hardware
  availability tests.

### 6. Tests Required

- Unit tests for HLS output codec classification.
- Profile validation tests for omitted/H264 accepted, HEVC/AV1 deferred, and
  unknown codec unsupported.
- Run `cargo nextest run -p nako-transcode hls --no-fail-fast` when changing
  this policy to prove command planning did not drift accidentally.

### 7. Wrong vs Correct

#### Wrong

```rust
if codec == "hevc" {
    encoder = "hevc_nvenc";
}
```

#### Correct

```rust
let output = HlsVideoOutputPolicyDecision::from_requested_codec(video_codec);
if !output.is_executable() {
    return Err(deferred_or_unsupported_output_error(output));
}
```

Keep output codec policy explicit before wiring hardware-specific encoder
execution.

## Scenario: HLS Runtime Subtitle Strategy Boundary

### 1. Scope / Trigger

- Trigger: changing playback-to-transcode HLS runtime planning fields,
  especially `TranscodeRequirement.subtitle_strategy`,
  `HlsRuntimePlanRequest.subtitle_strategy`, or HLS media rendition selection.

### 2. Signatures

- Playback emits `nako_playback::TranscodeRequirement.subtitle_strategy`.
- Server maps it through `playback_subtitle_strategy_to_transcode(...)`.
- Transcode consumes it through
  `nako_transcode::HlsRuntimePlanRequest.subtitle_strategy`.

### 3. Contracts

- Playback is the authority for subtitle intent. For HLS with a selected
  subtitle, supported subtitle delivery maps to `SidecarSelected`; unsupported
  delivery maps to `BurnInSelected`; no selected subtitle maps to `None`.
- Non-HLS transcode output may continue using `OmitSelected` until that output
  shape gets an explicit executable contract.
- HLS runtime planning must generate subtitle media renditions only when the
  request strategy is `SidecarSelected`.
- `BurnInSelected`, `OmitSelected`, and `None` must not create subtitle
  sidecar artifacts from `track_selection.subtitle_stream`.
- Audio media renditions are independent of subtitle strategy and must still be
  planned from probe/source facts when multi-audio HLS needs them.
- Transcode profile identity must include `subtitle_strategy`; request variant
  identity should include subtitle media renditions only when sidecar artifacts
  are actually planned.
- The FFmpeg HLS adapter supports `BurnInSelected` only for selected embedded
  text subtitles on the software filter pipeline. It must keep image subtitle,
  external subtitle, and hardware-filter burn-in behavior explicit rather than
  silently falling back.

### 4. Validation & Error Matrix

| Condition | Behavior |
|-----------|----------|
| HLS selected subtitle and target supports subtitle delivery | `SidecarSelected`; subtitle media rendition may be planned |
| HLS selected subtitle and target does not support subtitle delivery | `BurnInSelected`; no subtitle sidecar rendition |
| `OmitSelected` with `track_selection.subtitle_stream = Some(_)` | keep `OmitSelected`; no subtitle sidecar rendition |
| Multi-audio probe with non-sidecar subtitle strategy | audio renditions still appear; subtitle renditions do not |
| FFmpeg HLS request with executable text-subtitle `BurnInSelected` | primary subtitle burn-in filter, no subtitle sidecar output |
| FFmpeg HLS request with unsupported burn-in shape | invalid input or unsupported adapter error |

### 5. Good / Base / Bad Cases

- Good: server copies playback's typed subtitle strategy into
  `HlsRuntimePlanRequest`, and transcode branches media-rendition planning on
  that strategy.
- Base: `SidecarSelected` plus source subtitle facts yields subtitle sidecar
  media rendition identity.
- Bad: any HLS runtime code sees `track_selection.subtitle_stream.is_some()`
  and upgrades the execution policy to `SidecarSelected`.

### 6. Tests Required

- Playback unit test proving unsupported subtitle delivery selects
  `BurnInSelected`.
- Playback or server test proving supported HLS subtitle delivery reaches
  `SidecarSelected`.
- Transcode runtime test proving `BurnInSelected` preserves profile identity
  and creates no subtitle media renditions.
- Transcode runtime test proving `OmitSelected` is not upgraded to sidecar from
  track selection, while audio renditions are preserved.
- FFmpeg HLS command test proving `BurnInSelected` emits a primary subtitle
  burn-in filter and no subtitle sidecar output.
- Server HLS flow test proving supported subtitle delivery still publishes
  sidecar playlists and WebVTT artifacts.

### 7. Wrong vs Correct

#### Wrong

```rust
let mut pipeline = self.plan_hls_single_variant(pipeline_request, report)?;
if media_renditions.has_subtitles() {
    pipeline.subtitle_strategy = TranscodeSubtitleStrategy::SidecarSelected;
}
```

This rebuilds intent from planned artifacts and loses the distinction between
burn-in, omit, and sidecar requests.

#### Correct

```rust
pipeline_request.subtitle_strategy = request.subtitle_strategy;
let media_renditions = match request.subtitle_strategy {
    TranscodeSubtitleStrategy::SidecarSelected => selected_subtitle_and_audio_renditions()?,
    _ => selected_audio_renditions_only()?,
};
```

The playback decision stays the source of truth, and HLS runtime planning only
materializes artifacts allowed by that strategy.

## Gate Selection

- Focused transcode:
  `cargo nextest run -p nako-transcode <filter> --no-fail-fast`
- Playback/transcode/server:
  `cargo check -p nako-playback -p nako-transcode -p nako-server --tests`

## Review Checklist

- Is every FFmpeg argument typed and tested?
- Are artifacts represented by manifests?
- Are hardware and runtime budgets explicit?
- Does playback selection remain outside this crate?
