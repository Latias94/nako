# Server Architecture Hardening Milestones

## M24.0: Server Architecture Baseline

Outcome: the target boundaries are documented before Rust code is moved.

Deliverables:

- ADR 0019 for server composition, service, supervisor, and repository
  boundaries.
- This workstream with milestone, TODO, and phase baseline documents.
- Audit notes for the current `TaruApp`, service modules, background tasks,
  repository usage, NFO parsing, catalog hydration, and obsolete helpers.

Exit criteria:

- `git diff --check`

## M24.1: App Service Decomposition

Outcome: `TaruApp` becomes a composition root and workflow logic moves into
focused application services.

Deliverables:

- Explicit service handles for playback, metadata, NFO, library scan/probe,
  storage, catalog hydration, extension dispatch, and runtime maintenance.
- Route handlers call application services rather than broad root-app methods.
- Shared test fixtures construct services through the same composition path as
  the server.

Exit criteria:

- `cargo fmt --all -- --check`
- `cargo check --workspace --tests`
- focused `cargo nextest run -p taru-server`
- `git diff --check`

## M24.2: Repository and Transaction Boundary Cleanup

Outcome: high-level services depend on narrow contracts and multi-record
updates have explicit atomic boundaries.

Deliverables:

- Remove avoidable direct dependence on broad `SqliteStore` surfaces from
  feature orchestration.
- Move remaining multi-record write sequences into repository operations or an
  explicit unit-of-work boundary.
- Document any concrete-store dependency that remains intentionally server
  local.

Exit criteria:

- `cargo fmt --all -- --check`
- `cargo check --workspace --tests`
- focused `cargo nextest run -p taru-db`
- focused `cargo nextest run -p taru-server`
- `git diff --check`

## M24.3: Runtime Supervisor and Lifecycle Ownership

Outcome: background tasks have one lifecycle owner instead of feature-local
detached spawns.

Deliverables:

- Runtime supervisor or worker registry with startup, shutdown, cancellation,
  task handles, and diagnostics hooks.
- Metadata, NFO, staging cleanup, webhook, automation, and maintenance loops
  register through the supervisor when they need background execution.
- Tests cover cancellation and task failure reporting for at least one worker
  class.

Exit criteria:

- `cargo fmt --all -- --check`
- `cargo check --workspace --tests`
- focused `cargo nextest run -p taru-server`
- `git diff --check`

## M24.4: Domain Cleanup and Obsolete Code Removal

Outcome: early MVP shortcuts are removed after their replacement invariants are
available.

Deliverables:

- Remove source-to-library inference helpers that are obsolete after
  `MediaSource.library_id`.
- Replace hand-written NFO XML walking with a structured parser boundary.
- Remove duplicate config translation paths and route helper wrappers that no
  longer carry compatibility value.
- Collapse temporary service methods left behind by earlier decomposition.

Exit criteria:

- `cargo fmt --all -- --check`
- `cargo check --workspace --tests`
- focused `cargo nextest run -p taru-nfo`
- focused `cargo nextest run -p taru-server`
- `git diff --check`

## M24.5: Stabilization and Validation

Outcome: M24 closes with a small, enforceable architecture contract.

Deliverables:

- Phase close-out note mapping M24 goals to code, tests, and removed obsolete
  paths.
- Updated development docs for adding new application services and workers.
- Validation evidence for server, DB, NFO, metadata, catalog, and playback
  boundaries.

Exit criteria:

- `cargo fmt --all -- --check`
- `cargo check --workspace --tests`
- `cargo nextest run --workspace`
- `git diff --check`
