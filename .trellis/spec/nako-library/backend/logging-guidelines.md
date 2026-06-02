# Logging Guidelines

Library diagnostics should be useful without exposing host-local details.

## Rules

- Prefer durable ingestion failure records and summaries over ad hoc logs.
- Do not log raw local filesystem paths, credentials, provider payloads, or full
  source fingerprints.
- When diagnostics are needed, prefer library ID, job ID, source ID, phase,
  retryability, failure class, and redaction-safe message.
- High-volume scan/probe logs should stay behind deliberate tracing decisions.

## Evidence

- `crates/nako-library/src/failure.rs`
- `crates/nako-library/src/summary.rs`
- `docs/architecture/LIBRARY_PIPELINE.md`
