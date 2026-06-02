# Directory Structure

`nako-media-probe` currently fits in `src/lib.rs`: a public probe abstraction,
one ffprobe adapter, private JSON DTOs, and private mapping helpers.

## Current Layout

- `MediaProbeRequest`: VFS source plus optional local path hint.
- `MediaProbe`: async trait used by library/probe orchestration.
- `FfprobeMediaProbe`: configurable ffprobe executable path.
- `parse_ffprobe_json`: converts ffprobe JSON bytes into core probe facts.
- `stream_to_info`: maps a stream DTO into `MediaStreamInfo`.
- Private helpers for stream kind, numeric parsing, rational parsing, rotation,
  HDR metadata, and dynamic range.
- Private `Ffprobe*` DTO structs matching only consumed ffprobe JSON fields.
- Unit tests embedded in `src/lib.rs`.

## Module Rules

- Keep private ffprobe JSON structs near the mapper while the file is small.
- Split into modules only when adding a second probe backend or when mapping
  helpers become hard to review.
- Keep the public trait and request type at the crate root.
- Keep probe result/domain types in `nako-core`.
- Keep source staging and remote storage access outside this crate.

## Naming Rules

- Use `Ffprobe*` prefixes for adapter-specific DTOs and helpers.
- Use `MediaProbe*` names for backend-agnostic public API.
- Keep provider strings stable and lowercase, currently `ffprobe`.
- Name parse helpers by target type: `parse_u64`, `parse_rational`,
  `parse_seconds_to_ms`.

## Anti-Patterns

- Do not make ffprobe DTOs public.
- Do not add database, library scan, or VFS cache modules here.
- Do not embed library ingestion failure types in this crate.
- Do not hide probe backend choice inside global state.
