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
- FAD-050 — Playback/transcode request and cache identity.
- FAD-060 — Hardware acceleration diagnostics.

Current executable task:

- FAD-070 — Search semantics.

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

## FAD-050 Summary

FAD-050 stabilized playback/transcode request identity before widening HLS or
profile reuse:

- `taru-streaming` now exposes `PlaybackProfileIdentity` while keeping
  `PlaybackProfile::identity_key()` for compatibility.
- `taru-transcode` now models:
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

- `cargo check -p taru-streaming -p taru-transcode -p taru-server --tests`
- `cargo nextest run -p taru-transcode transcode_request_identity
  --no-fail-fast`
- `cargo nextest run -p taru-streaming playback_profile_identity
  --no-fail-fast`
- `cargo nextest run -p taru-server hls_source_request_identity
  --no-fail-fast`
- `cargo nextest run -p taru-streaming -p taru-transcode --no-fail-fast`
- `cargo nextest run -p taru-server playback --no-fail-fast`
- `cargo fmt --all -- --check`
- `git diff --check`

## FAD-060 Summary

FAD-060 separated the **Hardware Capability Report** diagnostics layers:

- `taru-transcode` now models:
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

- `cargo check -p taru-transcode -p taru-api -p taru-server --tests`
- `cargo nextest run -p taru-transcode hardware --no-fail-fast`
- `cargo nextest run -p taru-transcode --no-fail-fast`
- `cargo nextest run -p taru-api --lib
  admin_playback_runtime_diagnostics_serializes_safe_summary_fields
  --no-fail-fast`
- `cargo nextest run -p taru-api --lib admin_contract --no-fail-fast`
- `cargo nextest run -p taru-server
  admin_v1_playback_runtime_reports_safe_diagnostics --no-fail-fast`
- `npm run check` in `apps/admin-web`
- `cargo fmt --all -- --check`
- `git diff --check`

## Blockers

- None for FAD-070.

## Next Recommended Action

1. Start FAD-070.
2. Inspect `taru-search`, `taru-catalog`, and search projection callers.
3. Add a small search semantics evaluation harness for title, alias,
   provider-title, and CJK-friendly query behavior.
4. Keep AI/vector search out of this workstream unless split into a separate
   product lane.
