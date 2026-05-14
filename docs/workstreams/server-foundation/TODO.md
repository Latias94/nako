# Server Foundation TODO

## Architecture

- [x] Define workspace crate layout and dependency direction.
- [x] Add top-level documentation index, roadmap, and goal map.
- [x] Add refactoring policy for modular workspace evolution.
- [x] Create initial `taru-core` domain model draft.
- [x] Decide default database target for MVP.
- [x] Define service trait boundaries for library, metadata, VFS, transcode,
      search, events, automation, and addons.
- [x] Add ADR index and first accepted/rejected decisions.
- [x] Define shared job cancellation and retry policy.
- [x] Define shared job state and resource-budget model.
- [x] Add observability conventions for async pipelines and background jobs.
- [x] Add persisted job lifecycle state for server runtime work.
- [x] Add persisted job input payloads for audit and future retry.
- [x] Add minimal HTTP API for health, libraries, sources, items, probes, and jobs.
- [x] Add pagination foundation for list endpoints.
- [x] Add structured logging initialization for the server binary.

## VFS and Storage

- [x] Draft `StorageBackend` and `VirtualFile` traits.
- [x] Define storage capability flags.
- [x] Define stable media locator format.
- [x] Design directory metadata cache.
- [x] Design byte-range media cache.
- [x] Add lightweight local file fingerprints for incremental scan state.
- [ ] Document local filesystem backend behavior.
- [ ] Document WebDAV/S3/rclone integration paths.
- [ ] Decide how hard links and soft links are represented for non-local
      storage backends.

## Metadata and NFO

- [x] Define canonical metadata fields.
- [x] Define external ID model for TMDB, Douban, Bangumi, IMDb, and local IDs.
- [x] Define provider priority and field-level lock policy.
- [x] Define raw provider response cache policy.
- [x] Define NFO import/export compatibility targets.
- [x] Implement TMDB movie search/fetch MVP.
- [x] Add metadata refresh service using merge policy and field locks.
- [x] Add persisted metadata refresh job input and summary.
- [x] Add HTTP and CLI triggers for single-item metadata refresh.
- [x] Add NFO file discovery and import/export jobs.
- [x] Design catalog graph for people, credits, tags, genres, collections,
      studios, and artwork.
- [x] Design NFO actor/director mapping into the catalog graph.
- [ ] Add TMDB series, season, and episode support.
- [ ] Add Douban provider MVP.
- [ ] Add Bangumi provider MVP.

## Library Profiles and Scraping Strategy

- [x] Decide that library presets are editable configuration templates, not
      hard content types.
- [x] Define `MediaDomain` for broad processing capabilities.
- [x] Define `LibraryPreset` for setup defaults.
- [x] Define `LibraryOptions` for scan, naming, local metadata, and refresh
      behavior.
- [x] Define `MetadataProfile` for local readers, provider order, image
      providers, language, country, and refresh mode.
- [x] Persist library options and metadata profiles in SQLite.
- [x] Resolve metadata refresh provider order from the effective library
      profile instead of hard-coding TMDB.
- [x] Add tests for disabled providers, provider order, missing-only refresh,
      full refresh, and locked fields.
- [x] Add multi-provider fallback when the first configured provider cannot
      handle an item.
- [x] Design search indexing strategy for catalog graph fields.
- [ ] Add item-level metadata profile overrides.

## Catalog Graph and Artwork

- [x] Define people and item credit relationship model.
- [x] Define user tags, provider genres, collections, and studios as separate
      catalog concepts.
- [x] Define artwork cache and preview-generation resource classes.
- [x] Persist people, credits, tags, genres, collections, studios, and image
      assets in SQLite.
- [x] Persist artwork task queue records with resource class, status, attempts,
      and max-attempt retry state.
- [x] Teach metadata providers and NFO import to upsert catalog graph records.
- [x] Rebuild search projection after metadata refresh and NFO import.
- [x] Add browse APIs for item detail, credits, images, people, tags, and
      genres.
- [ ] Add image proxy/cache routes with etag and variant support.
- [ ] Add thumbnail and preview-frame generation jobs.

## Search

- [x] Define internal search adapter boundary.
- [x] Define initial catalog search document shape.
- [x] Define search update triggers from scan, metadata refresh, NFO import, and
      user edits.
- [x] Implement SQLite search projection fallback behind `SearchIndex`.
- [ ] Upgrade SQLite fallback to FTS ranking/tokenization when bundled FTS
      support is guaranteed.
- [ ] Add item/person/tag/genre search filters.
- [ ] Add optional Tantivy or Meilisearch adapter boundary after SQLite FTS.

## Scan State and Ingestion

- [x] Persist scan snapshots and directory snapshots.
- [x] Persist source state with fingerprint, last-seen scan, and tombstone flag.
- [x] Rebuild search projection when scan ingestion creates or updates an item.
- [ ] Add scan failure table for isolated per-directory/per-object errors.
- [ ] Add rename/move detection using strong fingerprints when available.

## Streaming and Transcoding

- [x] Define media probe output model.
- [x] Persist media probe output and stream details.
- [x] Define bounded probe pipeline and default concurrency.
- [x] Define playback decision model.
- [x] Define direct play, remux, and transcode decision rules.
- [x] Add source-level playback decision API.
- [x] Add direct play HTTP route with byte-range support.
- [x] Move direct play response planning behind streaming and app boundaries.
- [x] Add direct play HEAD preflight and range edge-case coverage.
- [x] Draft FFmpeg command builder interface.
- [x] Add remux session manager skeleton.
- [x] Implement FFmpeg process runner for remux sessions.
- [x] Add remux process cancellation, timeout, concurrency guard, and temp cleanup.
- [x] Add remux application service in `taru-server::app`.
- [x] Define local remux staging directory policy.
- [x] Define duplicate remux request reuse or idempotency behavior.
- [x] Map remux runner errors into stable application/API errors.
- [x] Add HTTP remux playback route backed by the remux app service.
- [x] Persist remux/transcode session records.
- [ ] Implement HLS transcode session manager.
- [ ] Define hardware acceleration detection model for VAAPI, NVENC, and QSV.
- [ ] Define CPU/GPU transcode concurrency and queue policy beyond remux.
- [ ] Define remote source staging/cache behavior for FFmpeg.

## Addons and Automation

- [ ] Draft Taru addon manifest schema.
- [ ] Define addon resource routes and response envelopes.
- [ ] Define addon timeout, retry, authentication, and trust model.
- [ ] Define automation job model.
- [x] Define API-key provider secret storage requirements.
- [ ] Define webhook event envelope and delivery policy.

## Documentation

- [x] Create MVP milestone document.
- [x] Create initial API design notes.
- [x] Create top-level roadmap and goal tracker.
- [x] Create workstream index.
- [x] Create fearless refactoring policy.
- [x] Create local development setup guide.
- [x] Create test strategy for crate-level and integration tests.
- [x] Create licensing notes for reference-only GPL code.
