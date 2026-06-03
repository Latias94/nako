# Staging Budget Per-Backend Policy

## Goal

Move staging-pressure policy beyond the current global manifest total by adding
a bounded per-backend or per-library policy slice that can support safer
operator diagnostics and future admission decisions.

## MVP Scope

- Audit existing staging manifest records, storage backend keys, and Admin
  staging diagnostics.
- Add the smallest policy representation that can distinguish staging pressure
  by backend/library without a broad schema rewrite unless one is clearly
  required.
- Preserve the current global pressure summary and Admin DTO compatibility.
- Add tests for local and WebDAV-backed staging pressure attribution.

## Out of Scope

- No scan scheduler fairness changes; lane B owns scheduler behavior.
- No watcher/debounce behavior.
- No Public Client API change.
- No raw Source Locator, local path, fingerprint, credential, or backend URL in
  diagnostics.

## Acceptance Criteria

- [x] Per-backend/library staging pressure policy is represented through a typed
  boundary.
- [x] Existing global staging diagnostics remain compatible.
- [x] Tests prove pressure attribution does not leak redacted data.
- [x] Deferred follow-ons are recorded if schema, PostgreSQL, or scheduler
  changes are intentionally skipped.

## Suggested Gates

- `cargo check -p nako-server --tests`
- Focused `cargo nextest run -p nako-server <new filters> --no-fail-fast`
- `cargo fmt --all -- --check`
- `git diff --check`

## Implementation Notes

- The shipped MVP derives typed staging policy slices from existing staging
  manifest records instead of adding a schema migration.
- `policy_slices` were added to Admin staging diagnostics while preserving the
  existing global summary fields and pressure thresholds.
- Synchronous remote scan admission now uses the matching library/backend slice;
  queued scheduler global staging pressure behavior remains unchanged and is
  intentionally left for `05b`.

## Deferred Follow-Ons

- Persisted per-library staging attribution for overlapping or same-root
  multi-endpoint libraries.
- PostgreSQL/runtime parity evidence for derived staging policy slices.
- Scan scheduler fairness and library-aware queued scheduling under mixed local
  and remote pressure.
