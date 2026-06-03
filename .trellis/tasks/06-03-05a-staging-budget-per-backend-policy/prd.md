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

- [ ] Per-backend/library staging pressure policy is represented through a typed
  boundary.
- [ ] Existing global staging diagnostics remain compatible.
- [ ] Tests prove pressure attribution does not leak redacted data.
- [ ] Deferred follow-ons are recorded if schema, PostgreSQL, or scheduler
  changes are intentionally skipped.

## Suggested Gates

- `cargo check -p nako-server --tests`
- Focused `cargo nextest run -p nako-server <new filters> --no-fail-fast`
- `cargo fmt --all -- --check`
- `git diff --check`
