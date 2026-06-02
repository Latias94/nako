# Directory Structure

`nako-transcode` owns transcode/remux/HLS command and artifact planning. It does
not decide whether playback should transcode; that decision comes from
`nako-playback`.

## Current Layout

```text
crates/nako-transcode/src/
├── lib.rs                 # public exports and crate tests
├── plan.rs                # generic transcode plan validation
├── policy.rs              # transcode execution policy
├── hardware.rs            # hardware acceleration report/capabilities
├── profile.rs             # profile values
├── probe.rs               # ffprobe/inventory-related values
├── artifact.rs            # artifact/session IDs and manifests
├── hls.rs                 # HLS artifact requirements and manifests
├── remux.rs               # remux execution planning
├── runtime.rs             # runtime limits, permits, cancellation primitive
├── execution.rs           # execution plan records
├── engine.rs              # engine adapter kind
├── progress.rs            # progress parsing/state
├── runner_util.rs         # command helper utilities
└── ffmpeg/                # FFmpeg command builders and HLS/remux modules
```

## Module Rules

- Keep FFmpeg argument construction inside `ffmpeg/*`.
- Keep HLS manifest/artifact shape in `hls.rs` and `artifact.rs`.
- Keep generic plan validation in `plan.rs`.
- Keep runtime concurrency/timeout/cancellation primitives in `runtime.rs`.
- Keep hardware capability inventory and policy values separate from command
  builders.
- Re-export public planning values from `lib.rs`; keep test-only FFmpeg command
  internals behind `#[cfg(test)]`.

## Forbidden Placement

- Do not choose playback mode here. `nako-playback` owns Direct Play/Remux/
  Transcode/Denied planning.
- Do not serve HLS playlists, media segments, direct byte streams, or playback
  tickets here. Server/streaming crates own transport.
- Do not perform server admission/resource queueing here. `nako-server` owns
  playback runtime admission.
- Do not store transcode sessions in the database here.

## Examples

- `ffmpeg/hls.rs`: HLS command planning through typed request values.
- `runtime.rs`: `TranscodeRuntimeGuard` with concurrency and timeout limits.
- `plan.rs`: HLS codec validation for playback transcode plans.
- `lib.rs` tests: command argv assertions for remux/HLS planning.
