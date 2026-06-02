# Logging Guidelines

Provider diagnostics must be useful to operators without leaking secrets.

## Rules

- Do not log provider API keys, bearer tokens, signed URLs, raw request headers,
  or full raw provider payloads.
- Prefer structured attempt/diagnostic records over ad hoc logs for provider
  refresh behavior.
- Log or record provider name, subject kind, capability status, and safe error
  class when diagnostics are needed.
- Keep Admin/Public summaries redacted; detailed provider payload inspection
  requires a deliberate diagnostics contract.

## Evidence

- `crates/nako-metadata/src/provider_attempt.rs`
- `crates/nako-metadata/src/runtime.rs`
- ADR 0018
