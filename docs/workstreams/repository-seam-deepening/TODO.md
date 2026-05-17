# Repository Seam Deepening Task Ledger

Status: Completed
Last updated: 2026-05-17

## Tasks

- [x] RSD-010 [owner=codex] [scope=docs/workstreams/repository-seam-deepening]
  Goal: Open M39 with workflow-port scope, first slice, non-goals, and gates.
  Validation: `git diff --check`.
  Handoff: Continue with `CatalogHydrationPort`.

- [x] RSD-020 [owner=codex] [deps=RSD-010] [scope=crates/taru-catalog, crates/taru-core, crates/taru-db]
  Goal: Extract `CatalogHydrationPort` and make catalog hydration depend on
  the workflow port rather than the wide `CatalogRepository + MediaRepository
  + SearchIndex` combination.
  Validation: `cargo check -p taru-catalog --tests`, `cargo nextest run -p
  taru-catalog --no-fail-fast` with 3 tests passed.
  Handoff: Preserve existing `SqliteStore` catalog behavior.

- [x] RSD-030 [owner=codex] [deps=RSD-020] [scope=crates/taru-metadata, crates/taru-nfo]
  Goal: Narrow metadata refresh, hierarchy confirmation, and NFO import bounds
  to use catalog hydration through the new port.
  Validation: `cargo check -p taru-metadata --tests`, `cargo nextest run -p
  taru-metadata --no-fail-fast`, `cargo check -p taru-nfo --tests`, `cargo
  nextest run -p taru-nfo --no-fail-fast`; metadata passed 26 tests and NFO
  passed 8 tests.
  Handoff: Do not change provider breadth or NFO codec behavior.

- [x] RSD-040 [owner=codex] [deps=RSD-030] [scope=docs]
  Goal: Update GOALS, ROADMAP, workstream index, and evidence.
  Validation: `git diff --check`.
  Handoff: Split follow-ons for metadata refresh or library scan/probe ports.

- [x] RSD-050 [owner=codex] [deps=RSD-040] [scope=workspace]
  Goal: Close M39 with focused and workspace gates.
  Validation: `cargo fmt --all -- --check`, focused crate checks/nextest,
  `cargo check --workspace --tests`, `cargo nextest run --workspace
  --no-fail-fast`, `git diff --check`.
  Handoff: Recommend the next port extraction only after M39 evidence is clear.
