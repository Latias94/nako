# nako-streaming Backend Guidelines

`nako-streaming` owns direct byte/range response planning and simple content
type helpers. It does not execute playback decisions, transcode jobs, or HTTP
server wiring.

## Current Evidence

- `crates/nako-streaming/src/direct.rs`
- `crates/nako-streaming/src/lib.rs`
- `docs/architecture/PLAYBACK.md`

## Boundaries

- Parse HTTP range headers.
- Resolve satisfiable byte ranges against object length.
- Plan direct-play response status, headers, and body range metadata.
- Keep actual byte transport in server or storage adapters.
- Keep playback policy in `nako-playback` and FFmpeg planning in
  `nako-transcode`.

## Required Patterns

- Keep planners pure and deterministic.
- Return `RangeNotSatisfiable` plans for malformed or out-of-bounds ranges.
- Use `bytes */len` content-range syntax for unsatisfiable ranges.
- Preserve full-object responses when no range is requested.
- Use filename-based content type helper only as a simple fallback.

## Forbidden Patterns

- Do not read files or streams from this crate.
- Do not spawn FFmpeg or transcode work.
- Do not create HTTP framework response types here.
- Do not panic on malformed range headers.

## Validation

- Focused:
  `cargo nextest run -p nako-streaming --no-fail-fast`
- Playback contract:
  `cargo check -p nako-streaming -p nako-playback --tests`
