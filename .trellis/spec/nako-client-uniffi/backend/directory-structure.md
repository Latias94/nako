# Directory Structure

`nako-client-uniffi` currently fits in `src/lib.rs` because it is a mirror of
`nako-client-core`.

## Current Layout

- UniFFI records for headers, request inputs, requests, responses, safe previews,
  failures, and connection probe results.
- UniFFI enums for playback modes, output containers, failure kinds, and probe
  outcome kinds.
- Exported functions delegating to core builders.
- `From` conversions between UniFFI and core types.
- `uniffi::setup_scaffolding!()`.
- Focused tests for exported request builders.

## Module Rules

- Keep mirror type definitions near exported functions while the file is
  manageable.
- Split conversion helpers only if type count becomes hard to review.
- Keep all route logic in `nako-client-core`.
- Keep generated binding artifacts out of source unless a release workflow
  explicitly adds them.

## Naming Rules

- Use the same `Core*` names as `nako-client-core` mirror types.
- Use exported function names matching the core builder names.
- Use `From` conversions rather than ad hoc conversion helpers.

## Anti-Patterns

- Do not create async client methods here.
- Do not add CLI or reqwest types.
- Do not duplicate core request-building implementation.
