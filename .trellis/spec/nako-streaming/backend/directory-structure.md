# Directory Structure

`nako-streaming` keeps direct response planning in a small module.

## Current Layout

- `direct.rs`: range parsing, byte-range resolution, direct-play response plan,
  and content type helper.
- `lib.rs`: public exports.

## Module Rules

- Keep range parsing and response planning together while the crate stays small.
- Add a new module only for a real streaming mode with its own pure planning
  contract.
- Keep HTTP framework adapters outside this crate.
- Keep storage reads outside this crate.

## Naming Rules

- Use `DirectPlay*` for direct byte-serving plans.
- Use `ByteRange` for resolved inclusive ranges.
- Use `RangeNotSatisfiable` for invalid or unsatisfiable range responses.

## Anti-Patterns

- Do not add `axum`, `hyper`, or `tokio` response glue here.
- Do not create local-file-specific modules.
- Do not mix direct streaming with HLS/transcode artifact planning.
