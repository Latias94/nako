# Error Handling

Transcode errors should identify planning, validation, runtime guard, and
external engine failures without leaking unsafe paths or command payloads.

## Required Patterns

- Validate playback transcode plans through typed validation errors before
  command execution.
- Reject in-place remux/transcode outputs.
- Runtime guard acquisition failures map to `NakoError::Provider` for the
  FFmpeg engine.
- Cancellation should use the typed cancellation primitive and caller-owned
  lifecycle; do not rely on dropped futures as the cancellation contract.
- Hardware inventory probe errors should mark inventory degraded rather than
  claiming full readiness.

## Validation Matrix

| Condition | Behavior |
|-----------|----------|
| Empty input locator | invalid input plan validation |
| HLS video codec not h264 | plan validation error |
| HLS audio codec not aac | plan validation error |
| Input and output path are same | reject command plan |
| Runtime semaphore closed | `NakoError::Provider { provider: "ffmpeg", ... }` |
| Hardware probe error | degraded inventory status |

## Wrong vs Correct

### Wrong

```rust
let args = format!("-i {} {}", input.display(), output.display());
```

### Correct

```rust
let command = FfmpegCommandBuilder::new("ffmpeg").hls(&request)?;
```

Use typed command builders so paths and options stay structured and testable.

## Evidence

- `crates/nako-transcode/src/plan.rs`
- `crates/nako-transcode/src/runtime.rs`
- `crates/nako-transcode/src/lib.rs`
