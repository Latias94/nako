# Fearless Architecture Deepening — Handoff

Status: Completed
Last updated: 2026-05-20

## Current State

M62 PostgreSQL Production Readiness has been committed as
`e45fa1a refactor: complete postgresql production readiness`.

This workstream is now the active architecture-deepening lane for the next
fearless refactor pass. It records the 2026-05-20 architecture review findings
and prioritizes the Modules most likely to hurt future Nako evolution if they
harden as-is.

Completed tasks:

- FAD-020 — Addon Side Effect Module depth.
- FAD-030 — Addon metadata commit atomicity.
- FAD-040 — Library ingestion workflow depth.
- FAD-050 — Playback/transcode request and cache identity.
- FAD-060 — Hardware acceleration diagnostics.
- FAD-070 — Search semantics.
- FAD-080 — Test locality for touched search semantics tests.
- FAD-090 — Final closeout.

Current executable task:

- None. The workstream is closed.

Why FAD-020 comes first:

- Addon Side Effects touch permission, grants, redaction, idempotency, storage,
  Canonical Metadata authority, Catalog Item Graph/Search Projection refresh,
  NFO/Library File Write policy, artwork candidate intake, and future plugin
  safety.
- `crates/nako-server/src/app/addons.rs` currently concentrates too many of
  those concerns in one Module.
- A behavior-preserving split can improve locality before semantic changes.

## Decisions So Far

- Keep the lane architecture-first. Do not add provider breadth, network
  traversal, native plugin ABI, adaptive bitrate, or AI runtime features here.
- Managed Artwork PostgreSQL parity was split to
  `docs/workstreams/managed-artwork-postgresql-parity/` and later completed.
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
- `library_file_write.rs` for NFO Library File Write apply through Nako's NFO
  service, VFS backend, write policy, and backup policy.
- `artwork_write.rs` for Addon Artwork Candidate proposal.
- `target.rs` for shared Media Item resolution from side-effect targets.

Validation passed:

- `cargo check -p nako-server --tests`
- `cargo nextest run -p nako-server addon_side_effect --no-fail-fast`
- `cargo nextest run -p nako-server addon --no-fail-fast`
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
- `nako-catalog` now has planning helpers for search-only projection and
  label-focused graph projection so server code can plan before persistence.
- `metadata_write.rs` no longer sequences `commit_metadata_item` plus
  catalog/search mutation plus later apply-outcome recording. It builds the
  domain commit and delegates atomicity to `nako-db`.
- The Addon Side Effect apply router now returns the side-effect outcome already
  recorded by the metadata commit seam for metadata writes.

Validation passed:

- `cargo check -p nako-core -p nako-db -p nako-server --tests`
- `cargo nextest run -p nako-db addon_metadata_write --no-fail-fast`
- `cargo nextest run -p nako-server addon_side_effect --no-fail-fast`
- `cargo fmt --all -- --check`
- `git diff --check`

PostgreSQL opt-in:

- Not run because `NAKO_TEST_POSTGRES_URL` was unset.
- Contract pair exists and should be run when a PostgreSQL test URL is
  available.

## FAD-040 Summary

FAD-040 introduced a Library ingestion workflow seam:

- Deleted the caller-facing `LibraryIndexRepository` broad trait alias.
- Added `LibraryIngestionWorkflow` as the Nako Library ingestion port.
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

- `cargo check -p nako-library -p nako-db --tests`
- `cargo nextest run -p nako-library
  index_service_uses_workflow_port_without_repository_traits --no-fail-fast`
- `cargo nextest run -p nako-db scan_commit --no-fail-fast`
- `cargo nextest run -p nako-library --no-fail-fast`
- `cargo fmt --all -- --check`
- `git diff --check`

PostgreSQL opt-in:

- Not run because `NAKO_TEST_POSTGRES_URL` was unset.
- Existing PostgreSQL scan commit contract pair remains available as ignored
  opt-in parity coverage.

## FAD-050 Summary

FAD-050 stabilized playback/transcode request identity before widening HLS or
profile reuse:

- `nako-streaming` now exposes `PlaybackProfileIdentity` while keeping
  `PlaybackProfile::identity_key()` for compatibility.
- `nako-transcode` now models:
  - `TranscodeProfileIdentity` for execution/profile facts;
  - `TranscodeSourceIdentity` for Media Source revision facts;
  - `TranscodeRequestIdentity` for the source-bound request/cache key.
- Remux and HLS app services now use `TranscodeRequestIdentity` for persisted
  session request keys, in-flight duplicate keys, finished-output reuse, and
  staging output slugs.
- Source revision changes now create a different HLS/remux cache/session
  identity even when the source id and playback profile are unchanged.
- Selected hardware acceleration remains part of the HLS profile identity, so
  CPU fallback and GPU runs do not reuse each other's output.
- No adaptive bitrate ladder, subtitles, HDR/SDR variant behavior, or
  multi-profile HLS reuse was added.

Validation passed:

- `cargo check -p nako-streaming -p nako-transcode -p nako-server --tests`
- `cargo nextest run -p nako-transcode transcode_request_identity
  --no-fail-fast`
- `cargo nextest run -p nako-streaming playback_profile_identity
  --no-fail-fast`
- `cargo nextest run -p nako-server hls_source_request_identity
  --no-fail-fast`
- `cargo nextest run -p nako-streaming -p nako-transcode --no-fail-fast`
- `cargo nextest run -p nako-server playback --no-fail-fast`
- `cargo fmt --all -- --check`
- `git diff --check`

## FAD-060 Summary

FAD-060 separated the **Hardware Capability Report** diagnostics layers:

- `nako-transcode` now models:
  - `HardwareEncoderDiscovery` for static FFmpeg encoder discovery;
  - `HardwareDeviceInitialization` for device initialization evidence;
  - `HardwareSmokeProbe` for optional encode smoke-probe results.
- Default runtime detection still uses `ffmpeg -encoders`; normal tests do not
  open privileged GPU devices or require host hardware access.
- A new `HardwareDeviceInitializationDetector` seam plus static fake detector
  lets tests prove device initialization pass/fail outcomes separately from
  encoder discovery and smoke probes.
- Explicit device-initialization failures and smoke-probe failures make an
  accelerator unavailable. Missing encoders and FFmpeg probe errors remain
  distinct diagnostic reasons.
- Admin playback runtime diagnostics now expose safe per-accelerator summaries:
  encoder discovery status and encoder name, device initialization status and
  operator guidance, smoke-probe status and operator guidance, and `has_detail`
  booleans. Raw FFmpeg errors, local paths, device paths, and probe detail text
  are not returned.
- Admin TypeScript contract and admin-web mock playback runtime data were
  updated to the new shape.

Validation passed:

- `cargo check -p nako-transcode -p nako-api -p nako-server --tests`
- `cargo nextest run -p nako-transcode hardware --no-fail-fast`
- `cargo nextest run -p nako-transcode --no-fail-fast`
- `cargo nextest run -p nako-api --lib
  admin_playback_runtime_diagnostics_serializes_safe_summary_fields
  --no-fail-fast`
- `cargo nextest run -p nako-api --lib admin_contract --no-fail-fast`
- `cargo nextest run -p nako-server
  admin_v1_playback_runtime_reports_safe_diagnostics --no-fail-fast`
- `npm run check` in `apps/admin-web`
- `cargo fmt --all -- --check`
- `git diff --check`


## FAD-070 Summary

FAD-070 added measured search semantics without adding AI/vector search:

- `nako-search` now owns shared search evaluation with:
  - current Search Projection version helpers;
  - `SearchEvaluationDocument` fixtures;
  - exact Browse Facet filtering;
  - title, alias, body, and facet scoring;
  - compact normalized matching for whitespace-tolerant CJK queries.
- SQLite and PostgreSQL `SearchIndex` adapters now delegate search semantics to
  `nako-search` instead of duplicating filtering/scoring logic.
- Catalog hydration now includes accepted Provider Subject title/key data in the
  Search Projection, so Provider Mapping titles are searchable as aliases while
  provider/external-id Browse Facets remain structured.
- `NfoImportRepository` now includes `ProviderMappingRepository` because richer
  Catalog Projection planning needs accepted Provider Mapping context.
- No AI, vector search, FTS backend, pinyin, romaji, provider breadth, or
  external search service was added.

Validation passed:

- `cargo check -p nako-search -p nako-catalog -p nako-db --tests`
- `cargo check -p nako-nfo -p nako-metadata -p nako-server --tests`
- `cargo nextest run -p nako-search --no-fail-fast`
- `cargo nextest run -p nako-catalog semantic_search --no-fail-fast`
- `cargo nextest run -p nako-db search --no-fail-fast`
- `cargo nextest run -p nako-db facet --no-fail-fast`
- `cargo fmt --all -- --check`
- `git diff --check`

Environment note:

- `C:\Users\Frankorz\AppData\Local\Temp` had no free space during this
  slice. Final link/test commands were run with `TMP` and `TEMP` set to
  `F:\Temp`.

## FAD-080 Summary

FAD-080 improved test locality only where it clearly reduced navigation cost:

- Moved the focused SQLite SearchIndex semantics tests out of the giant
  `crates/nako-db/src/tests.rs` file into
  `crates/nako-db/src/search_tests.rs`.
- Added local domain fixtures for migrated stores, Movie Canonical Metadata, and
  indexed search documents with explicit aliases/facets.
- Preserved coverage and failure meaning for:
  - exact Browse Facet filtering;
  - CJK-friendly alias lookup through the shared search semantics;
  - alias search while keeping aliases structured.
- Left broader mixed-purpose DB and server HTTP/app test families untouched
  where splitting would be mechanical churn rather than a better test boundary.

Validation passed:

- `cargo nextest run -p nako-db search --no-fail-fast`
- `cargo nextest run -p nako-db facet --no-fail-fast`
- `cargo check --workspace --tests`
- `cargo fmt --all -- --check`
- `git diff --check`

Environment note:

- Link/check commands used `TMP` and `TEMP` set to `F:\Temp` because the default
  user temp directory had no free space.

## Blockers

- None.

## Next Recommended Action

1. Continue the already-active `admin-api-typescript-contract` lane if Admin
   Web contract drift is the next priority.
2. Pick up `managed-artwork-postgresql-parity` when Managed Artwork PostgreSQL
   runtime enablement is needed.
3. Open a new product workstream for provider breadth, AI/vector search,
   network traversal, adaptive playback, or client UX rather than reopening
   M63.

## FAD-090 Summary

FAD-090 closed M63 after final verification.

Validation passed:

- `cargo fmt --all -- --check`
- `cargo check --workspace --tests`
- `cargo nextest run --workspace --no-fail-fast` — 498 passed, 19 skipped
- `git diff --check`

PostgreSQL opt-in:

- Skipped because `NAKO_TEST_POSTGRES_URL` was unset in this environment.

Closeout split decision:

- No new follow-on workstream was created from this closeout. The remaining
  independent tails are already named:
  - `managed-artwork-postgresql-parity`
  - `admin-api-typescript-contract`
