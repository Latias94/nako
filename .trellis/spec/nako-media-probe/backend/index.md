# nako-media-probe Backend Guidelines

`nako-media-probe` extracts technical media facts from VFS-backed sources. The
current implementation is an `ffprobe` adapter that requires a local path hint
and maps JSON into `nako-core` media probe records.

## Pre-Development Checklist

- Read [Directory Structure](./directory-structure.md) before adding probe
  implementations, parsing helpers, or modules.
- Read [Database Guidelines](./database-guidelines.md) before introducing any
  persistence or cache behavior.
- Read [Error Handling](./error-handling.md) before changing process execution,
  JSON parsing, unsupported source handling, or provider error messages.
- Read [Quality Guidelines](./quality-guidelines.md) before changing ffprobe
  mapping, media technical facts, HDR detection, rational parsing, or tests.
- Read [Logging Guidelines](./logging-guidelines.md) before adding probe
  diagnostics.

## Guidelines Index

| Guide | Description | Status |
|-------|-------------|--------|
| [Directory Structure](./directory-structure.md) | Probe trait, ffprobe adapter, JSON model, mapping helpers | Filled from code |
| [Database Guidelines](./database-guidelines.md) | No-persistence probe boundary | Filled from code |
| [Error Handling](./error-handling.md) | Provider, unsupported, and parse failure mapping | Filled from code |
| [Quality Guidelines](./quality-guidelines.md) | Deterministic ffprobe fact mapping and test cases | Filled from code |
| [Logging Guidelines](./logging-guidelines.md) | Redaction-safe probe diagnostics | Filled from code |

## Authority / Evidence

- `crates/nako-media-probe/src/lib.rs`
- `crates/nako-media-probe/Cargo.toml`
- `crates/nako-core/src/lib.rs`
- `crates/nako-vfs/src/lib.rs`

## Boundaries

- Own `MediaProbeRequest`, `MediaProbe`, and probe adapter implementations.
- Execute external probe tools through controlled async process calls.
- Map probe output into `MediaProbeResult`, `MediaStreamInfo`, and
  `MediaStreamTechnicalFacts`.
- Keep storage access in VFS/library layers; this crate consumes
  `StorageUri` plus optional local path hints.
- Keep durable ingestion failure persistence in `nako-library`.

## Executable Contract Summary

1. Scope / Trigger: probe adapters, ffprobe JSON mapping, media technical fact
   extraction, HDR detection, or local-path requirements update this crate.
2. Signatures: `MediaProbe::probe`, `MediaProbeRequest`, and
   `FfprobeMediaProbe`.
3. Contracts: current ffprobe adapter requires `local_path_hint`; it invokes
   `ffprobe -v error -print_format json -show_format -show_streams`.
4. Validation & Error Matrix: missing local path returns
   `NakoError::Unsupported`; process or parse failures return
   `NakoError::Provider` with provider `ffprobe`.
5. Good/Base/Bad Cases: good mapping preserves stream kind, codec, duration,
   bit rate, language, color, HDR, rotation, rational frame rates, and
   disposition; bad mapping silently drops known ffprobe fields without tests.
6. Tests Required: JSON mapping tests for video/audio/subtitle, HDR, rotation,
   disposition, rational parsing, and unsupported local path behavior.
7. Wrong vs Correct: do not add VFS fetching or ingestion persistence here; pass
   a staged local path from the library/VFS workflow.

## Validation

- Focused:
  `cargo nextest run -p nako-media-probe --no-fail-fast`
- Cross-crate contracts:
  `cargo check -p nako-media-probe -p nako-library --tests`
