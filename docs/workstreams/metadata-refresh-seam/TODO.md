# Metadata Refresh Seam Task Ledger

Status: Completed
Last updated: 2026-05-17

## Tasks

- [x] MRS-010 [owner=codex] [scope=docs/workstreams/metadata-refresh-seam]
  Goal: Open M40 with refresh seam scope, first slice, non-goals, and gates.
  Validation: `git diff --check`.
  Handoff: Continue with refresh strategy/repository audit.

- [x] MRS-020 [owner=codex] [deps=MRS-010] [scope=crates/taru-metadata, crates/taru-core]
  Goal: Audit metadata refresh and hierarchy confirmation repository/provider
  dependencies, then choose the first workflow-port shape.
  Validation: documented `MetadataRefreshPort` and `MetadataAttemptPort` in
  `DESIGN.md`.
  Handoff: Do not implement provider breadth.

- [x] MRS-030 [owner=codex] [deps=MRS-020] [scope=crates/taru-metadata]
  Goal: Extract the first metadata refresh workflow port and make the refresh
  strategy depend on it where the interface hides real persistence detail.
  Validation: `cargo check -p taru-metadata --tests`.
  Handoff: Preserve `CatalogHydrationPort` usage.

- [x] MRS-040 [owner=codex] [deps=MRS-030] [scope=crates/taru-metadata]
  Goal: Add a focused fake-port test that exercises the refresh workflow
  without SQLite while keeping existing SQLite-backed tests intact.
  Validation: `cargo nextest run -p taru-metadata --no-fail-fast` with 27
  tests passed.
  Handoff: Record whether provider runtime should be a follow-on slice.

- [x] MRS-050 [owner=codex] [deps=MRS-040] [scope=docs]
  Goal: Update GOALS, ROADMAP, workstream index, and evidence.
  Validation: `git diff --check`.
  Handoff: Split follow-ons for provider runtime or library scan/probe seams.

- [x] MRS-060 [owner=codex] [deps=MRS-050] [scope=workspace]
  Goal: Close M40 with focused and workspace gates.
  Validation: `cargo fmt --all -- --check`, focused crate checks/nextest,
  `cargo check --workspace --tests`, `cargo nextest run --workspace
  --no-fail-fast`, `git diff --check`.
  Handoff: Recommend the next seam only after M40 evidence is clear.
