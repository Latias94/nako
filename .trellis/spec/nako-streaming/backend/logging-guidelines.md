# Logging Guidelines

`nako-streaming` should usually return response plans rather than log. If
future diagnostics are added, keep them small and transport-neutral.

## Required Patterns

- Prefer counters for requested range kind, response status, object length, and
  resolved range length.
- Avoid logging raw URLs or authorization-sensitive locators.
- Keep malformed range diagnostics concise.
- Let server middleware own request IDs and client network metadata.

## Forbidden Patterns

- Do not log full storage URIs with credentials.
- Do not log one line per byte chunk from this crate.
- Do not add HTTP request logging here.
- Do not use logs as the only place where range errors are visible.

## Useful Fields

- `streaming.range_kind`
- `streaming.status`
- `streaming.object_length`
- `streaming.range_start`
- `streaming.range_end`
