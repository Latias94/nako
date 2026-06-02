# Logging Guidelines

`nako-db` should keep database diagnostics redaction-safe and adapter-focused.

## Rules

- Do not log raw SQL with bound values when those values may include tokens,
  secrets, local paths, provider cache payloads, or playback tickets.
- Prefer returning structured domain diagnostics through repository records
  when users or operators need to see state.
- If tracing is added around database work, use feature/action names and record
  counts, not sensitive payloads.
- Migration failures should surface through `NakoError::Database`; do not add
  side-channel logs that leak connection strings.

## Examples

- `contract_tests.rs` asserts repository behavior instead of relying on logs.
- `sqlite/codec.rs` converts persisted values into domain types without logging
  raw rows.
