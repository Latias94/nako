# nako-transcode Backend Development Guidelines

These specs describe FFmpeg command planning, remux/HLS/transcode artifact
modeling, hardware capability inventory, and transcode runtime primitives in
`crates/nako-transcode`.

## Pre-Development Checklist

- Read [Directory Structure](./directory-structure.md) before adding FFmpeg,
  HLS, remux, hardware, runtime, or artifact modules.
- Read [Database Guidelines](./database-guidelines.md) before touching
  transcode session persistence from related work.
- Read [Error Handling](./error-handling.md) before changing plan validation,
  command planning, runtime guard, or cancellation behavior.
- Read [Quality Guidelines](./quality-guidelines.md) before changing FFmpeg
  arguments, HLS artifact manifests, hardware policy, or runtime limits.
- Read [Logging Guidelines](./logging-guidelines.md) before adding FFmpeg or
  transcode diagnostics.

## Guidelines Index

| Guide | Description | Status |
|-------|-------------|--------|
| [Directory Structure](./directory-structure.md) | FFmpeg, HLS, remux, hardware, artifact, runtime modules | Filled from code and ADRs |
| [Database Guidelines](./database-guidelines.md) | Transcode runtime persistence non-ownership | Filled as adapter boundary |
| [Error Handling](./error-handling.md) | Plan validation, FFmpeg planning errors, runtime guard failures | Filled from code |
| [Quality Guidelines](./quality-guidelines.md) | Typed FFmpeg planning, artifact manifests, hardware/runtime gates | Filled from code |
| [Logging Guidelines](./logging-guidelines.md) | Redaction-safe FFmpeg/transcode diagnostics | Filled from code |

## Authority / Evidence

- ADR 0045: FFmpeg hardware pipeline planner.
- ADR 0049: source-aware transcode runtime.
- ADR 0052: HLS runtime and media engine boundary.
- `docs/architecture/PLAYBACK.md`
- `crates/nako-transcode/src/lib.rs`
- `crates/nako-transcode/src/ffmpeg/*`
- `crates/nako-transcode/src/hls.rs`
- `crates/nako-transcode/src/runtime.rs`
