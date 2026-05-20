# Managed Artwork PostgreSQL Parity — TODO

Status: Proposed
Last updated: 2026-05-20

Task IDs use the `MAPG` prefix.

## M0 — Scope And Split Baseline

- [ ] MAPG-010 [owner=planner] [deps=PGR-090] [scope=docs/workstreams/managed-artwork-postgresql-parity,docs/workstreams/postgresql-production-readiness]
  Goal: Confirm the split from M62, inventory existing Managed Artwork SQLite
  schema/repository/runtime behavior, and decide the first contract slice.
  Validation: `rg -n "ManagedArtwork|ArtworkCandidate|selected_artwork|managed_artwork|addon_artwork" crates/taru-core crates/taru-db crates/taru-server docs/workstreams`; `git diff --check`.
  Review: Do not enable PostgreSQL Managed Artwork runtime until a safe support
  boundary is implemented.
  Evidence: inventory in `EVIDENCE_AND_GATES.md`.
  Handoff: Continue with MAPG-020 contract slice.

## M1 — Repository Contract Slices

- [ ] MAPG-020 [owner=codex] [deps=MAPG-010] [scope=crates/taru-db,crates/taru-core,docs/workstreams/managed-artwork-postgresql-parity]
  Goal: Add backend-neutral contracts for Addon Artwork Candidate intake and
  Managed Artwork candidate acceptance/ingest queue creation.
  Validation: focused DB artwork candidate/ingest nextest for SQLite and
  PostgreSQL opt-in; `cargo check -p taru-db --tests`; `git diff --check`.
  Review: Candidate source details remain internal and redacted.
  Evidence: contract tests and PostgreSQL migration parity.
  Handoff: Continue with ingest claim/commit/fail/requeue contracts.

- [ ] MAPG-030 [owner=codex] [deps=MAPG-020] [scope=crates/taru-db,crates/taru-server/src/app/artwork.rs,docs/workstreams/managed-artwork-postgresql-parity]
  Goal: Add parity for Managed Artwork ingest claim, artifact commit, fail,
  startup recovery, and requeue behavior.
  Validation: focused DB/server managed artwork ingest tests for SQLite and
  PostgreSQL opt-in; `cargo check -p taru-db --tests`; `cargo check -p taru-server --tests`; `git diff --check`.
  Review: Job ownership and artifact commit ordering must remain fenced and
  atomic.
  Evidence: contract tests, server tests, migration parity.
  Handoff: Continue with Selected Artwork publication/gallery contracts.

- [ ] MAPG-040 [owner=codex] [deps=MAPG-030] [scope=crates/taru-db,crates/taru-server/src/app/artwork.rs,docs/workstreams/managed-artwork-postgresql-parity]
  Goal: Add parity for Selected Artwork publish/unpublish, gallery snapshots,
  and lifecycle cleanup candidates.
  Validation: focused selected artwork/gallery/lifecycle nextest for DB/server;
  PostgreSQL opt-in contract run; `git diff --check`.
  Review: Public/Admin responses must not leak storage URI, managed-artwork URI,
  local path, source URL, cache URI, or content hash.
  Evidence: redaction tests and migration parity.
  Handoff: Continue with runtime enablement/diagnostics.

## M2 — Runtime Support Boundary

- [ ] MAPG-050 [owner=codex] [deps=MAPG-040] [scope=crates/taru-server,crates/taru-api,docs/api,docs/workstreams/managed-artwork-postgresql-parity]
  Goal: Either enable Managed Artwork on PostgreSQL end-to-end or add explicit
  safe diagnostics/route gating until all required parity is proven.
  Validation: focused admin/public artwork route tests; `cargo check -p taru-api --tests`; `cargo check -p taru-server --tests`; `git diff --check`.
  Review: No partial PostgreSQL route/worker enablement with SQLite-only state.
  Evidence: runtime tests and docs.
  Handoff: Close or split remaining image-processing/diagnostic tails.
