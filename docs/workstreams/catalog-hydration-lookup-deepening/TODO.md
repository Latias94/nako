# Catalog Hydration Lookup Deepening Task Ledger

Status: Completed
Last updated: 2026-05-17

## M0 - Scope And Evidence Freeze

- [x] CHLD-010 [owner=codex] [deps=none] [scope=docs/workstreams/catalog-hydration-lookup-deepening]
  Goal: Open M42 with problem, target state, non-goals, and evidence anchors.
  Validation: Workstream docs exist and agree.
  Evidence: `docs/workstreams/catalog-hydration-lookup-deepening/DESIGN.md`
  Handoff: Continue with the seam-shape refactor.

## M1 - Workflow-Level Port

- [x] CHLD-020 [owner=codex] [deps=CHLD-010] [scope=crates/taru-catalog/src/lib.rs]
  Goal: Change `CatalogHydrationPort` to expose workflow-level hydration while
  keeping snapshot/lookup/commit machinery internal to `taru-catalog`.
  Validation: `cargo check -p taru-catalog --tests`
  passed.
  Evidence: Non-catalog callers now call `hydrate_catalog`, and catalog tests
  still pass.
  Handoff: Completed; metadata fake tests were updated in CHLD-030.

- [x] CHLD-030 [owner=codex] [deps=CHLD-020] [scope=crates/taru-metadata/src/strategy.rs,crates/taru-metadata/src/confirmation.rs,crates/taru-nfo/src/import.rs]
  Goal: Update metadata and NFO callers/tests to use the deeper hydration
  port without constructing lookup internals.
  Validation: `cargo nextest run -p taru-metadata strategy::port_tests::refresh_service_uses_refresh_and_hydration_ports_without_sqlite --no-fail-fast`
  passed.
  Evidence: Metadata fake port returns a hydration summary instead of
  snapshot/lookup/commit records.
  Handoff: Completed; NFO and workspace validation passed as well.

## M2 - Validation And Closeout

- [x] CHLD-040 [owner=codex] [deps=CHLD-030] [scope=workspace,docs]
  Goal: Update top-level docs and close M42 with focused and workspace gates.
  Validation: `cargo fmt --all -- --check`; `cargo check --workspace --tests`;
  `cargo nextest run --workspace --no-fail-fast`; `git diff --check`.
  passed.
  Evidence: `EVIDENCE_AND_GATES.md`
  Handoff: Recommend the next correctness or architecture goal after M42.
