# Runtime Foundation Milestones

## M15: Runtime Foundation Hardening

Outcome: Taru's shared runtime foundation is explicit, safe under expected
single-process concurrency, and no longer depends on MVP shortcuts that leak
across feature boundaries.

Deliverables:

- SQLite runtime configuration with WAL, busy timeout, bounded pool settings,
  and documented transaction expectations.
- Migration execution that does not split SQL with naive string parsing.
- Unified secret wrapper/redaction policy for provider config, server config,
  resolved secrets, job inputs, diagnostics, and provider structs.
- Playback runtime path that uses hardware capability detection and selected
  acceleration instead of directly consuming the requested accelerator.
- Updated workstream documentation and validation checklist.

Exit criteria:

- `cargo fmt --all -- --check`
- `cargo check --workspace --tests`
- `cargo nextest run --workspace`
- `git diff --check`

## M15.1: SQLite Runtime And Migrations

Outcome: database access behaves predictably under concurrent local server
workloads and migrations are safe for future complex SQL.

Candidate deliverables:

- Explicit SQLite connection options for WAL, foreign keys, busy timeout, and
  create-if-missing behavior.
- Bounded connection pool defaults with a clear read/write concurrency policy.
- Migration runner based on a robust executor or constrained migration format,
  not `split(';')`.
- Migration tests covering strings containing semicolons and transactional
  rollback behavior.
- Documentation for SQLite operational limits in the current single-process
  architecture.

## M15.2: Secret Redaction Boundary

Outcome: resolved secrets cannot accidentally cross logging, debugging, API, or
job boundaries as plain strings.

Candidate deliverables:

- `SecretString` or `Redacted<T>` type with safe `Debug`, optional serde
  behavior, and explicit reveal methods.
- Provider config refactor for TMDB, Bangumi, Douban, and future providers.
- Server config refactor for provider headers, API keys, access tokens,
  proxies, and secret environment references.
- Tests proving `Debug` and diagnostics redact resolved values.
- Removal of provider-specific ad hoc redaction once the shared type is in use.

## M15.3: Hardware Capability Selection

Outcome: HLS/transcode sessions use a selected runtime acceleration decision
with fallback semantics and resource budgets.

Candidate deliverables:

- Server-side hardware capability probe boundary.
- `SelectedAcceleration` or equivalent value passed into HLS/transcode planning.
- Fallback behavior for unavailable accelerators controlled by config policy.
- Resource budget selection based on the selected accelerator, not the
  requested accelerator.
- Tests for CPU-only, available GPU, unavailable-with-CPU-fallback, and
  unavailable-with-fail-policy cases.

Implemented in M15:

- HLS uses the selected accelerator for command planning and resource budgets.
- The current server report is CPU-only; real FFmpeg probing is deferred to a
  later hardware enablement phase.
- Tests cover available GPU detector selection, CPU fallback, and fail-policy
  behavior in the server runtime path.

## M15.4: Runtime Module Decomposition

Outcome: runtime policy code is split into focused modules when a phase touches
the area, instead of growing monolithic app or library files.

Candidate deliverables:

- `taru-db` runtime and migration modules split from repository
  implementations.
- Metadata provider configuration/building split from maintenance planning.
- Playback hardware/runtime policy split from HLS session orchestration.
- New API surfaces use explicit DTOs and avoid expanding direct core model
  exposure.
