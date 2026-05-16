# Phase 27.1: Catalog Schema and Repository Slice

## Status

Completed implementation slice.

## Objective

Turn the M27.0 metadata-catalog domain baseline into durable `taru-core`
records, `taru-db` schema, repository traits, SQLite adapters, and focused
repository tests without adding provider breadth or public API behavior.

## Implemented Scope

M27.1 added three persistence slices:

- **Provider Subject** and **Provider Mapping** records persist provider
  evidence separately from Taru **Media Item** identity.
- **Source Duplicate Relationship** records persist duplicate-source evidence
  separately from source identity and item merging.
- **Local Inference Evidence** records persist scanner inference evidence for
  source-local kind, title, year, season, episode, confidence, evidence source,
  and inference version.

The slice intentionally keeps the existing `MediaSource.item_id` primary link.
One item may still have multiple sources, and duplicate-source relationships
do not rewrite source-to-item ownership.

## Code Changes

- `crates/taru-core/src/id.rs` adds IDs for provider subjects, provider
  mappings, source duplicate relationships, and local inference evidence.
- `crates/taru-core/src/media.rs` adds records and enums for:
  - `ProviderSubject`, `ProviderSubjectKind`
  - `ProviderMapping`, `ProviderMappingStatus`
  - `SourceDuplicateRelationship`, `SourceDuplicateEvidenceKind`,
    `SourceDuplicateRelationshipStatus`
  - `LocalInferenceEvidence`, `LocalInferenceEvidenceSource`
- `crates/taru-core/src/repository.rs` adds repository traits for provider
  mappings, source duplicate relationships, and local inference evidence.
- `crates/taru-db/migrations/0018_metadata_catalog_domain.sql` adds the
  normalized SQLite tables and indexes.
- `crates/taru-db/src/provider_mapping.rs` implements provider subject and
  provider mapping repository methods.
- `crates/taru-db/src/source_duplicate.rs` implements duplicate relationship
  repository methods and canonicalizes source pairs.
- `crates/taru-db/src/local_inference.rs` implements local inference evidence
  repository methods.
- `crates/taru-db/src/tests.rs` adds repository round-trip coverage for all
  three new slices, selected video item hierarchy, multi-source item links,
  and verifies that mappings/evidence do not replace item identity.

## Compatibility

Existing movie MVP behavior remains compatible:

- `MediaItem` and `MediaSource` round-trip tests still pass.
- Provider mappings do not replace `CanonicalMetadata.external_ids`; those
  compatibility fields remain available.
- Duplicate-source relationships do not merge sources or change
  `MediaSource.item_id`.
- Local inference evidence does not confirm metadata or change an unknown item
  into a confident item kind by itself.

## Validation

Commands run:

- `cargo nextest run -p taru-db sqlite_store_round_trips_provider_subjects_and_mappings`
- `cargo nextest run -p taru-db sqlite_store_round_trips_video_item_hierarchy_and_multiple_sources`
- `cargo nextest run -p taru-db sqlite_store_round_trips_source_duplicate_relationships_without_merging_items`
- `cargo nextest run -p taru-db sqlite_store_round_trips_local_inference_evidence_without_confirming_metadata`
- `cargo nextest run -p taru-db` - 31 passed
- `cargo nextest run -p taru-core` - 3 passed
- `cargo fmt --all -- --check`
- `cargo check --workspace --tests`
- `git diff --check` - passed with Git CRLF normalization warnings only

## Remaining Boundaries

M27.2 should build on this schema and remain focused on provider/NFO/artwork
expansion:

- TMDB series, season, and episode provider mapping.
- Douban provider MVP.
- Bangumi provider MVP.
- NFO round-trip preservation beyond the current movie subset renderer.
- Artwork candidate, selected artwork, and managed artwork contracts.

Client browse/sort DTOs should remain a later M27 slice unless the API names
and repository queries are explicitly designed first.
