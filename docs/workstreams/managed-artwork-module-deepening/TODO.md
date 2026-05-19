# Managed Artwork Module Deepening Task Ledger

Status: Active
Last updated: 2026-05-19

## M0 - Scope And Audit

- [x] MAMD-010 [owner=codex] [deps=none] [scope=docs/workstreams/managed-artwork-module-deepening,docs/workstreams/README.md]
  Goal: Open the lane with scope, non-goals, redaction invariants, module
  candidates, and validation gates.
  Validation: Workstream docs exist and agree; `WORKSTREAM.json` parses;
  `git diff --check`.
  Evidence: `DESIGN.md`, `EVIDENCE_AND_GATES.md`.
  Result: DONE. Lane opened with explicit non-goals, redaction invariants,
  module candidates, and low-concurrency validation gates.

## M1 - Variant Presentation Module

- [x] MAMD-020 [owner=codex] [deps=MAMD-010] [scope=crates/taru-server/src/app/artwork.rs,crates/taru-server/src/app/artwork/variant.rs]
  Goal: Extract Selected Artwork variant request policy, derivation, byte
  envelope, and presentation ETag behavior into a private app Module with no
  public behavior change.
  Validation: focused server/API image variant tests plus `cargo check` for
  touched packages.
  Evidence: variant tests still pass; no raw artifact storage fields appear in
  public/Admin variant responses or HTTP headers.
  Result: DONE. `artwork/variant.rs` now owns variant request validation,
  artifact media-type planning, original/derived byte envelope creation,
  on-demand resizing, and opaque presentation ETag generation. The app service
  keeps only orchestration and the existing crate re-export.

## M2 - Artifact Store Module

- [x] MAMD-030 [owner=codex] [deps=MAMD-020] [scope=crates/taru-server/src/app/artwork.rs,crates/taru-server/src/app/artwork/artifact_store.rs]
  Goal: Extract local Managed Artwork Artifact storage, inventory, file
  classification, and safe delete outcomes into a private artifact store
  Module.
  Validation: focused artifact lifecycle, drift inventory, remediation, and
  ingest artifact tests.
  Evidence: callers no longer assemble local artifact paths or parse storage
  layout outside the Module.
  Result: DONE. `artwork/artifact_store.rs` owns local artifact path layout,
  storage URI validation, write/read/delete operations, store inventory,
  discovered file parsing, and classified file issues. The app service now
  projects internal artifact store issues into Admin DTO reasons instead of
  coupling the store Module to `taru-api`.

## M3 - Ingest Pipeline Module

- [x] MAMD-040 [owner=codex] [deps=MAMD-030] [scope=crates/taru-server/src/app/artwork.rs,crates/taru-server/src/app/artwork/ingest_pipeline.rs]
  Goal: Extract fetch, validation, artifact write, and safe failure summary
  creation into an ingest pipeline Module while keeping durable job claiming
  and commit ordering in the app service.
  Validation: focused Managed Artwork ingest processing and failure redaction
  tests.
  Evidence: processing behavior remains unchanged; runtime retry/cancel
  semantics are not added.
  Result: DONE. `artwork/ingest_pipeline.rs` now owns remote fetch, content-type
  normalization, image validation, content hash creation, artifact file write
  preparation, success summary serialization, and safe failure summary
  serialization. `ManagedArtworkAppService` keeps durable claim, DB commit,
  best-effort artifact rollback after commit failure, and failure commit
  ordering.

## M4 - Repository Adapter Module Split

- [ ] MAMD-050 [owner=codex] [deps=MAMD-040] [scope=crates/taru-db/src/artwork.rs,crates/taru-db/src/artwork/**]
  Goal: Split SQLite Managed Artwork repository implementation by concern while
  preserving existing `taru-core` repository traits and public crate exports.
  Validation: focused `taru-db` artwork tests and repository compile checks.
  Evidence: SQL constants and row mappers move beside the concern they support.
  Progress: Gallery and lifecycle read-model SQL/constants/row mapping have
  moved to `crates/taru-db/src/artwork/gallery.rs` and
  `crates/taru-db/src/artwork/lifecycle.rs`. Trait impls still route through
  the same repository methods. Remaining work: split selected-artwork
  publication/unpublication and core ingest/artifact transaction helpers.
  Handoff: Continue DB adapter split with selection or ingest helper modules.

## M5 - API Surface Audit

- [ ] MAMD-060 [owner=codex] [deps=MAMD-050] [scope=crates/taru-api/src/admin.rs,crates/taru-api/src/admin/**,crates/taru-api/src/public_client.rs]
  Goal: Audit Managed Artwork DTO locality and split only if doing so reduces
  caller knowledge without weakening explicit DTO/redaction tests.
  Validation: Admin/Public Client redaction tests and OpenAPI checks.
  Evidence: redaction tests remain close to DTO conversion code.
  Handoff: Close or split remaining follow-ons.

## M6 - Closeout

- [ ] MAMD-070 [owner=codex] [deps=MAMD-060] [scope=docs/workstreams/managed-artwork-module-deepening,workspace]
  Goal: Verify the lane, record evidence, list intentional residual work, and
  close or split follow-ons.
  Validation: focused gates plus formatting and whitespace checks.
  Evidence: `EVIDENCE_AND_GATES.md`, `HANDOFF.md`, `WORKSTREAM.json`.
