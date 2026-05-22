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

- [x] MCC-020 [owner=codex] [deps=MCC-010] [scope=crates/nako-core,crates/nako-catalog,crates/nako-db]
  Goal: Replace the separate catalog graph and search projection write path
  with one catalog hydration commit interface.
  Validation: `cargo check -p nako-catalog --tests`; `cargo nextest run -p
  nako-catalog --no-fail-fast`; `cargo check -p nako-db --tests`; `cargo
  nextest run -p nako-db
  sqlite_store_rolls_back_catalog_graph_when_search_projection_commit_fails`.
  Evidence: `crates/nako-catalog/src/lib.rs`, `crates/nako-db/src/catalog.rs`.
  Handoff: Graph replacement and search projection now commit through
  `CatalogRepository::commit_item_projection`; existing standalone
  `SearchIndex::upsert` remains for scan/index callers.

## M2 - Metadata Refresh Persistence Commit Unit

- [x] MCC-030 [owner=codex] [deps=MCC-020] [scope=crates/nako-core,crates/nako-metadata,crates/nako-db]
  Goal: Fold Canonical Metadata, Provider Raw Response, Provider Subject,
  accepted Provider Mapping, and Library Item State confirmation into one
  metadata refresh persistence commit interface.
  Validation: `cargo check -p nako-db --tests`; `cargo check -p
  nako-metadata --tests`; `cargo check -p nako-server --tests`; `cargo
  nextest run -p nako-db commit_metadata_refresh --no-fail-fast`; `cargo
  nextest run -p nako-metadata --no-fail-fast`; `cargo fmt --all --
  --check`; `git diff --check`.
  Evidence: `crates/nako-core/src/media/metadata.rs`,
  `crates/nako-core/src/repository/metadata.rs`,
  `crates/nako-metadata/src/strategy.rs`,
  `crates/nako-db/src/metadata.rs`.
  Handoff: Metadata refresh persistence now commits through
  `MetadataRepository::commit_metadata_refresh`; the old shallow
  `apply_metadata_refresh` interface was removed. Catalog hydration remains a
  separate workflow step whose graph/search write is atomic after MCC-020.

## M3 - Closeout

- [x] MCC-040 [owner=codex] [deps=MCC-030] [scope=docs/workstreams/metadata-catalog-commit-atomicity]
  Goal: Close the lane after recording fresh closeout evidence and splitting
  any larger prepared-catalog or projection-pipeline follow-up out of scope.
  Validation: Fresh gate evidence recorded in `EVIDENCE_AND_GATES.md`.
  Evidence: `WORKSTREAM.json`, `HANDOFF.md`.
  Handoff: Route NFO merge policy work to `metadata-merge-policy-unification`
  and any larger metadata-refresh-plus-prepared-catalog design to the
  architecture review follow-up lane.
