# Quality Guidelines

Transcode changes must keep FFmpeg behavior typed, bounded, and testable.

## Required Patterns

- Build FFmpeg commands through typed request and command builder structs.
- Assert exact argv shape in unit tests for new command-planning behavior.
- Use artifact manifests before publishing HLS/remux outputs to callers.
- Keep overwrite policy explicit.
- Keep hardware acceleration policy and capability reports explicit; do not
  probe GPU support for every playback request.
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
| Burn-in/preserve-in-container subtitles | unsupported FFmpeg adapter error |

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
