# Quality Guidelines

Streaming changes must preserve RFC-style range semantics and keep direct
streaming independent from playback policy.

## Required Patterns

- Cover range parsing with table-driven tests.
- Keep inclusive range math clear and overflow-safe.
- Keep response plans serializable and framework-neutral.
- Preserve deterministic headers for full, partial, and unsatisfiable responses.
- Keep content type mapping simple and predictable.

## Forbidden Patterns

- Do not introduce external IO into range planning.
- Do not combine direct response planning with transcode artifact selection.
- Do not use lossy integer casts for lengths or offsets.
- Do not change malformed range behavior without tests.

## Tests Required

- Full content response tests.
- Open-ended range tests.
- Suffix range tests if supported by parser behavior.
- Malformed header tests.
- Out-of-bounds range tests.
- Content type mapping tests.

## Gate Selection

- Focused:
  `cargo nextest run -p nako-streaming --no-fail-fast`
- Cross-layer compile:
  `cargo check -p nako-streaming -p nako-server --tests`
