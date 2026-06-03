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
