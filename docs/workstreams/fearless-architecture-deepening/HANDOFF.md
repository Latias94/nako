# Fearless Architecture Deepening — Handoff

Status: Active
Last updated: 2026-05-20

## Current State

M62 PostgreSQL Production Readiness has been committed as
`e45fa1a refactor: complete postgresql production readiness`.

This workstream is now the active architecture-deepening lane for the next
fearless refactor pass. It records the 2026-05-20 architecture review findings
and prioritizes the Modules most likely to hurt future Taru evolution if they
harden as-is.

Completed tasks:

- FAD-020 — Addon Side Effect Module depth.
- FAD-030 — Addon metadata commit atomicity.
- FAD-040 — Library ingestion workflow depth.

Current executable task:

- FAD-050 — Playback/transcode request and cache identity.

Why FAD-020 comes first:

- Addon Side Effects touch permission, grants, redaction, idempotency, storage,
  Canonical Metadata authority, Catalog Item Graph/Search Projection refresh,
  NFO/Library File Write policy, artwork candidate intake, and future plugin
  safety.
- `crates/taru-server/src/app/addons.rs` currently concentrates too many of
  those concerns in one Module.
- A behavior-preserving split can improve locality before semantic changes.

## Decisions So Far

- Keep the lane architecture-first. Do not add provider breadth, network
  traversal, native plugin ABI, adaptive bitrate, or AI runtime features here.
- Managed Artwork PostgreSQL parity remains a separate proposed follow-on:
  `docs/workstreams/managed-artwork-postgresql-parity/`.
- Prefer deep workflow seams over mechanical trait splits.
- New persistence commit seams must prove SQLite and PostgreSQL behavior
  through backend-neutral contracts.
- Addon metadata write atomicity is the first semantic refactor after the
  behavior-preserving Addon Module split.

## FAD-020 Summary

FAD-020 split the Addon Side Effect implementation into focused server Modules:

- `principal.rs` for Addon Principal resolution, grant authorization, token
  label normalization, and grant normalization.
- `intake.rs` for side-effect idempotency, validation, safe validation error
  codes, and accepted/rejected intake persistence.
- `side_effect_apply.rs` for apply routing and common apply-outcome recording.
- `metadata_write.rs` for Canonical Metadata patch/merge plus the existing
  catalog/search refresh behavior.
- `library_file_write.rs` for NFO Library File Write apply through Taru's NFO
  service, VFS backend, write policy, and backup policy.
- `artwork_write.rs` for Addon Artwork Candidate proposal.
- `target.rs` for shared Media Item resolution from side-effect targets.

Validation passed:

- `cargo check -p taru-server --tests`
- `cargo nextest run -p taru-server addon_side_effect --no-fail-fast`
- `cargo nextest run -p taru-server addon --no-fail-fast`
- `cargo fmt --all -- --check`
- `git diff --check`

FAD-020 intentionally preserved behavior. It did not fix Addon metadata write
atomicity; that remains the purpose of FAD-030.

## FAD-030 Summary

FAD-030 introduced a transactional Addon Canonical Metadata write seam:

- Core now exposes `AddonMetadataWritePersistenceCommit` with item mutation,
  optional Catalog Item Graph replacement, Search Projection, Addon Side Effect
  id, applied source, and optional apply report.
- SQLite and PostgreSQL commit the item, graph/search projection, and Addon
  Side Effect `Applied` outcome inside one transaction.
- `taru-catalog` now has planning helpers for search-only projection and
  label-focused graph projection so server code can plan before persistence.
- `metadata_write.rs` no longer sequences `commit_metadata_item` plus
  catalog/search mutation plus later apply-outcome recording. It builds the
  domain commit and delegates atomicity to `taru-db`.
- The Addon Side Effect apply router now returns the side-effect outcome already
  recorded by the metadata commit seam for metadata writes.

Validation passed:

- `cargo check -p taru-core -p taru-db -p taru-server --tests`
- `cargo nextest run -p taru-db addon_metadata_write --no-fail-fast`
- `cargo nextest run -p taru-server addon_side_effect --no-fail-fast`
- `cargo fmt --all -- --check`
- `git diff --check`

PostgreSQL opt-in:

- Not run because `TARU_TEST_POSTGRES_URL` was unset.
- Contract pair exists and should be run when a PostgreSQL test URL is
  available.

## FAD-040 Summary

FAD-040 introduced a Library ingestion workflow seam:

- Deleted the caller-facing `LibraryIndexRepository` broad trait alias.
- Added `LibraryIngestionWorkflow` as the Taru Library ingestion port.
- `LibraryIndexService` now asks the workflow to:
  - ensure the Media Library exists;
  - begin and complete scan snapshots;
  - record scan failures;
  - commit directory observations;
  - commit source observations;
  - tombstone sources missing from complete non-stale scans.
- The workflow Adapter now owns the ordering that used to live in the index
  service:
  - Source Locator lookup and inserted/updated disposition;
  - Local Inference planning;
  - confirmed Canonical Metadata preservation;
  - Provisional Hierarchy reuse/creation;
  - Source State and Library Item State composition;
  - Local Inference Evidence persistence composition;
  - Search Projection planning;
  - scan failure resolution;
  - delegation to the existing atomic `commit_library_scan_source` seam.
- Added a fake workflow test to prove the index service no longer needs the
  low-level repository trait set.

Validation passed:

- `cargo check -p taru-library -p taru-db --tests`
- `cargo nextest run -p taru-library
  index_service_uses_workflow_port_without_repository_traits --no-fail-fast`
- `cargo nextest run -p taru-db scan_commit --no-fail-fast`
- `cargo nextest run -p taru-library --no-fail-fast`
- `cargo fmt --all -- --check`
- `git diff --check`

PostgreSQL opt-in:

- Not run because `TARU_TEST_POSTGRES_URL` was unset.
- Existing PostgreSQL scan commit contract pair remains available as ignored
  opt-in parity coverage.

## Blockers

- None for FAD-050.

## Next Recommended Action

1. Start FAD-050.
2. Inspect playback source selection and transcode profile/request identity
   code before broadening HLS reuse, subtitles, HDR/SDR variants, or adaptive
   ladders.
3. Define a stable identity Interface and tests without adding adaptive bitrate
   behavior in this task.
4. Continue to FAD-060 hardware diagnostics only after identity semantics are
   proven.
