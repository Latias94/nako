# Server Architecture Hardening TODO

## Baseline

- [x] Add ADR 0019 for server architecture hardening boundaries.
- [x] Add M9 workstream documentation.
- [x] Re-audit `crates/taru-server/src/app.rs` after the first code pass and
  record the target `TaruApp` public surface.
- [x] Map existing background task launch sites and classify them by worker
  owner.
- [x] Map concrete `SqliteStore` dependencies from app services and decide
  which ones need ports.

## Application Services

- [x] Define the service handle shape for playback, metadata, NFO, library
  scan/probe, storage, catalog, and extension workflows.
- [x] Move orchestration methods out of `TaruApp` into service modules.
- [x] Keep service constructors explicit and test-friendly.
- [x] Update HTTP handlers to call focused services instead of root-app
  convenience methods.
- [x] Update app and HTTP test fixtures to use the production composition path.

Completed service handles so far:

- [x] Catalog read service and HTTP routing.
- [x] Library administration service and HTTP routing.
- [x] Storage diagnostics service and HTTP routing.
- [x] NFO service and HTTP routing.
- [x] Addon, automation, and webhook extension services and HTTP routing.

Remaining service handles:

- [x] Metadata service.
- [x] Library scan/probe service.
- [x] Playback service.
- [x] Remove temporary root-app forwards used by CLI and tests.

## Repository Boundaries

- [x] Replace avoidable broad-store dependencies with narrow repository traits
  or focused ports.
- [x] Identify remaining multi-record write sequences outside repository
  transaction boundaries.
- [x] Move catalog hydration write atomicity behind an explicit repository or
  unit-of-work operation.
- [x] Document intentionally concrete SQLite dependencies that remain inside
  server composition.

## Runtime Ownership

- [x] Design the runtime supervisor or worker registry API.
- [x] Register metadata maintenance and provider cleanup work through the
  supervisor.
- [x] Register NFO jobs and staging cleanup through the supervisor.
- [x] Audit webhook and automation execution ownership: webhook delivery uses
  request-scoped `JoinSet`, automation enqueue is synchronous, and neither owns
  detached workers that need supervisor registration.
- [x] Add cancellation and task-failure tests.
- [x] Ensure shutdown waits for owned workers or cancels them explicitly.

## Domain Cleanup

- [x] Remove library inference helpers made obsolete by `MediaSource.library_id`.
- [x] Replace hand-written NFO XML walking with a structured parser boundary.
- [x] Remove deprecated single-library or compatibility config paths that no
  longer serve a migration purpose.
- [x] Collapse temporary service wrappers left after decomposition.
- [x] Keep public API DTOs explicit; do not leak mutable domain aggregates.

## Validation

- [x] Run `cargo fmt --all -- --check`.
- [x] Run `cargo check --workspace --tests`.
- [x] Run focused `cargo nextest` suites for changed crates during each phase.
- [x] Run `cargo nextest run --workspace` before M9 close-out.
- [x] Run `git diff --check`.
- [x] Record validation evidence in the relevant phase close-out note.
