# Metadata Catalog Commit Atomicity TODO

Status: Proposed
Last updated: 2026-05-18

## M0 - Scope

- [x] MCC-010 [owner=codex] [deps=none] [scope=docs/workstreams/metadata-catalog-commit-atomicity]
  Goal: Open the workstream with problem, target state, non-goals, and gates.
  Validation: `git diff --check`.
  Evidence: `docs/workstreams/metadata-catalog-commit-atomicity/DESIGN.md`.
  Handoff: Continue with MCC-020.

## M1 - Catalog Graph/Search Atomic Commit

- [x] MCC-020 [owner=codex] [deps=MCC-010] [scope=crates/taru-core,crates/taru-catalog,crates/taru-db]
  Goal: Replace the separate catalog graph and search projection write path
  with one catalog hydration commit interface.
  Validation: `cargo check -p taru-catalog --tests`; `cargo nextest run -p
  taru-catalog --no-fail-fast`; `cargo check -p taru-db --tests`; `cargo
  nextest run -p taru-db
  sqlite_store_rolls_back_catalog_graph_when_search_projection_commit_fails`.
  Evidence: `crates/taru-catalog/src/lib.rs`, `crates/taru-db/src/catalog.rs`.
  Handoff: Graph replacement and search projection now commit through
  `CatalogRepository::commit_item_projection`; existing standalone
  `SearchIndex::upsert` remains for scan/index callers.

## M2 - Metadata Refresh Persistence Commit Unit

- [x] MCC-030 [owner=codex] [deps=MCC-020] [scope=crates/taru-core,crates/taru-metadata,crates/taru-db]
  Goal: Fold Canonical Metadata, Provider Raw Response, Provider Subject,
  accepted Provider Mapping, and Library Item State confirmation into one
  metadata refresh persistence commit interface.
  Validation: `cargo check -p taru-db --tests`; `cargo check -p
  taru-metadata --tests`; `cargo check -p taru-server --tests`; `cargo
  nextest run -p taru-db commit_metadata_refresh --no-fail-fast`; `cargo
  nextest run -p taru-metadata --no-fail-fast`; `cargo fmt --all --
  --check`; `git diff --check`.
  Evidence: `crates/taru-core/src/media/metadata.rs`,
  `crates/taru-core/src/repository/metadata.rs`,
  `crates/taru-metadata/src/strategy.rs`,
  `crates/taru-db/src/metadata.rs`.
  Handoff: Metadata refresh persistence now commits through
  `MetadataRepository::commit_metadata_refresh`; the old shallow
  `apply_metadata_refresh` interface was removed. Catalog hydration remains a
  separate workflow step whose graph/search write is atomic after MCC-020.

## M3 - Closeout

- [ ] MCC-040 [owner=planner] [deps=MCC-030] [scope=docs/workstreams/metadata-catalog-commit-atomicity]
  Goal: Close or split remaining metadata unit-of-work work.
  Validation: Fresh gate evidence recorded in `EVIDENCE_AND_GATES.md`.
  Evidence: `WORKSTREAM.json`, `HANDOFF.md`.
  Handoff: Route NFO merge policy work to `metadata-merge-policy-unification`.
