# nako-playback Backend Development Guidelines

These specs describe the pure playback planning boundary in
`crates/nako-playback`. This crate selects Direct Play, Remux, Transcode, or
Denied from source, probe, target, policy, storage, and preference facts. It
does not execute FFmpeg or serve bytes.

## Pre-Development Checklist

- Read [Directory Structure](./directory-structure.md) before adding playback
  planning values or capability evaluation modules.
- Read [Database Guidelines](./database-guidelines.md) before touching playback
  session persistence from related work.
- Read [Error Handling](./error-handling.md) before changing denial/report
  semantics.
- Read [Quality Guidelines](./quality-guidelines.md) before changing Direct
  Play/Remux/Transcode decision behavior.
- Read [Logging Guidelines](./logging-guidelines.md) before adding diagnostics.

## Guidelines Index

| Guide | Description | Status |
|-------|-------------|--------|
| [Directory Structure](./directory-structure.md) | Pure planner modules and exported value types | Filled from code and ADRs |
| [Database Guidelines](./database-guidelines.md) | Playback planner persistence non-ownership | Filled as not-applicable boundary |
| [Error Handling](./error-handling.md) | Decision denial and compatibility report behavior | Filled from code |
| [Quality Guidelines](./quality-guidelines.md) | Planner purity, capability profile, track/HDR/audio rules | Filled from code |
| [Logging Guidelines](./logging-guidelines.md) | No-runtime/logging boundary | Filled as no-runtime boundary |

## Authority / Evidence

- ADR 0038: playback planning and transcode policy seams.
- ADR 0039: playback policy and renderer target boundary.
- ADR 0044: playback capability profile planner.
- `docs/architecture/PLAYBACK.md`
- `crates/nako-playback/src/lib.rs`
- `crates/nako-playback/src/capability.rs`
- `crates/nako-playback/src/values.rs`
