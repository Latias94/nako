# Runtime Foundation TODO

## M15.0 Baseline

- [x] Create runtime-foundation workstream.
- [x] Document fearless refactor policy for pre-release runtime boundaries.
- [x] Record initial risk areas: SQLite runtime, migration execution, secret
      redaction, and hardware capability selection.
- [ ] Create ADR for runtime foundation hardening boundaries if implementation
      decisions need long-lived rationale.

## SQLite Runtime And Migrations

- [x] Define SQLite connection defaults for on-disk and in-memory stores.
- [x] Enable WAL and busy timeout for on-disk SQLite databases.
- [x] Decide pool sizing and transaction expectations for scan, playback,
      metadata maintenance, automation, and webhook workers.
- [x] Replace naive SQL statement splitting in migration execution.
- [x] Add migration tests for semicolons in string literals or other complex
      migration shapes.
- [x] Document current single-process SQLite assumptions and non-goals.

## Secret Redaction

- [x] Add shared redacted secret type.
- [x] Refactor TMDB provider config to use the shared type instead of ad hoc
      `Debug` redaction.
- [x] Refactor Bangumi provider config to avoid exposing resolved access
      tokens through `Debug`.
- [x] Refactor Douban provider config and custom headers to avoid exposing
      API keys or header values through `Debug`.
- [x] Audit server config and diagnostics for resolved secret leakage.
- [x] Add tests for debug output, diagnostics, and job input payloads.
- [x] Remove obsolete legacy secret handling paths after the shared boundary is
      in place.

## Hardware Capability Selection

- [x] Add or wire a server-side hardware capability detector.
- [x] Use `select_hardware_acceleration` in the HLS/transcode runtime path.
- [x] Pass selected acceleration into command planning and resource budgeting.
- [x] Preserve stable failure behavior when configured fallback is `fail`.
- [x] Add tests for selected CPU, available GPU selection, CPU fallback, and
      unavailable accelerator failure.
- [ ] Replace the current CPU-only server detector with real FFmpeg capability
      probing when hardware acceleration is enabled beyond policy planning.

## Module Boundaries

- [x] Split touched runtime code into focused modules as part of each phase.
- [ ] Avoid growing `nako-server::app`, `nako-server::app::metadata`,
      `nako-server::app::playback`, and `nako-db::lib` when adding runtime
      policies.
- [ ] Use explicit DTOs for new public API routes.
- [x] Delete legacy helpers once their replacement is fully wired.

## M16 Storage Backend Registry And Lease Lifecycle

- [x] Keep `StorageBackendRegistry` owned by `NakoApp`.
- [x] Cache backend wrappers by `library_id`.
- [x] Route scan, probe, playback, remux/HLS staging, and NFO through the
      registry boundary.
- [x] Keep per-library remote stream and stage resource budgets attached to
      backend wrappers.
- [x] Track backend health counters on storage operations.
- [x] Expose sanitized storage backend diagnostics through explicit API DTOs.
- [x] Model staging reserved, staging, ready, leased, expired, deleted, and
      failed states.
- [x] Ensure cleanup protects active leases.
- [x] Release staged-input leases explicitly after playback and through a
      drop-time fallback.

## M19 Database Boundary Hardening

- [x] Keep `sqlx` plus repository traits as the database boundary; do not add
      SeaORM.
- [x] Move shared SQL encoding and row mapping out of `nako-db::lib`.
- [x] Move root repository tests out of `nako-db::lib`.
- [x] Split job, outbox, automation, webhook, and addon repositories into
      focused modules.
- [x] Add a repository-level scan transaction for item, source, and source
      state writes.
- [x] Add a repository-level metadata refresh transaction for item metadata and
      provider raw response writes.
- [x] Preserve and document SQLite runtime and migration hardening from M15.
- [x] Run full workspace close-out validation.
