# Metadata-Catalog TODO

## M27.0 Domain Baseline

- [x] Keep M27.0 design-only: no schema migrations, provider features, runtime
      behavior, or public API changes.
- [x] Decide whether ADR 0021 should become accepted before M27.1 schema work.
- [x] Audit current **Media Item**, **Media Source**, provider mapping, catalog
      graph, metadata, NFO, **Library File Write**, artwork, search, and VFS
      capability surfaces against `CONTEXT.md`.
- [x] Audit current library config against **Media Domain** and **Library
      Preset** rules.
- [x] Decide whether current `MediaKind` variants are enough for series,
      season, episode, **Episode-Like Item**, **Extra Item**,
      **Franchise Collection**, and unknown video items.
- [x] Decide the first persistent source-to-item link model for multi-source
      items.
- [x] Decide how to represent **Source Duplicate Relationship** separately
      from source identity and item merge behavior.
- [x] Define **Provider Subject** and **Provider Mapping** rules for TMDB,
      Douban, Bangumi, and future provider/addon evidence.
- [x] Define **Metadata Source Priority** for local edits, NFO, TMDB, Douban,
      Bangumi, and future addon contributions.
- [x] Separate **Canonical Metadata**, **Media Technical Facts**, **Library
      Item State**, and **User Playback State** in the target model.
- [x] Define **Genre** vs **Tag** handling, and separate **Review Rating**,
      **Content Rating**, and **User Rating** ownership.
- [x] Define **NFO Round Trip** rules for import/export and unknown-field
      preservation.
- [x] Define **Managed Artwork**, **Artwork Candidate**, and **Selected
      Artwork** storage/API expectations.
- [x] Define initial client **Browse Facets** and **Sort Keys**.
- [x] Move active metadata/catalog TODO items out of `server-foundation` into
      this workstream.
- [x] Record ADRs for any schema, API, or provider-boundary decision that
      should not be rediscovered later.
- [x] Produce an explicit handoff for M27.1/M27.2 implementation boundaries.

## M27.1 Catalog Schema and Repository Slice

- [x] Keep M27.1 focused on `taru-core` records, repository traits,
      `taru-db` migrations/adapters, and repository tests.
- [x] Add durable **Provider Subject** records and **Provider Mapping**
      repository coverage without adding provider breadth.
- [x] Add durable **Source Duplicate Relationship** records and repository
      coverage without automatic source or item merging.
- [x] Persist minimal **Local Inference Evidence** for inferred kind, title,
      year, season, episode, confidence, evidence source, and inference
      version.
- [x] Cover the selected video item hierarchy and multi-source item link
      behavior through repository tests.
- [x] Keep existing movie MVP media item/source behavior working.
- [x] Update metadata-catalog evidence or milestone notes with validation
      commands and remaining M27.2 boundaries.
- [x] Run M27.1 gates: `cargo fmt --all -- --check`,
      `cargo check --workspace --tests`, focused `cargo nextest run -p
      taru-db`, focused `cargo nextest run -p taru-core`, and `git diff
      --check`.

## M27.2 Local Inference and Provisional Hierarchy

- [x] Keep M27.2 focused on local inference and provisional hierarchy; do not
      add TMDB, Douban, Bangumi, NFO confirmation, Source Variant UI, or
      browse API behavior.
- [x] Extend `taru-naming` parsed-name output with confidence, evidence source,
      parser version, and inferred kind/title/year/season/episode fields.
- [x] Make weak local parsing create **Unknown Media Item** output instead of a
      confident movie guess.
- [x] Persist **Local Inference Evidence** from `taru-library` scanning.
- [x] Keep **Local Inference Evidence** as a current snapshot per source,
      evidence source, and parser version rather than an append-only scan log.
- [x] Prevent local inference rescan from overwriting canonical metadata after
      an item is confirmed out of provisional state.
- [x] Create provisional series, season, and episode parent items during
      scanning.
- [x] Preserve or introduce library-scoped provisional item membership so
      source-less series/season items do not collide across libraries.
- [x] Keep existing movie indexing behavior working for confident movie names.
- [x] Update metadata-catalog evidence or milestone notes with validation
      commands and remaining M27.3 boundaries.
- [x] Run M27.2 gates: `cargo fmt --all -- --check`,
      `cargo check --workspace --tests`, focused `cargo nextest run -p
      taru-naming`, focused `cargo nextest run -p taru-library`, focused
      `cargo nextest run -p taru-db`, and `git diff --check`.

## Provider Breadth

- [x] Add shared **Hierarchy Confirmation** service boundary for provider/NFO
      confirmation of provisional hierarchy.
- [x] Confirm provisional series, season, and episode items in place without
      replacing Taru `MediaItem` identity.
- [x] Write accepted **Provider Mapping** records when metadata provider
      refresh succeeds.
- [x] Connect NFO episode import to the shared confirmation service while
      preserving local/NFO authority.
- [x] Add TMDB series, season, and episode provider fetch support.
- [x] Keep Douban provider MVP inside the shared Provider Subject / Mapping
      boundary.
- [x] Keep Bangumi provider MVP inside the shared Provider Subject / Mapping
      boundary.
- [ ] Add item-level metadata profile overrides.

## NFO and Library File Writes

- [ ] Harden NFO round-trip behavior before broad export support.
- [ ] Define how **Library File Write** behavior uses VFS write/link
      capabilities for NFO, artwork, subtitle, and sidecar writes across local
      and non-local backends.
- [ ] Define mediated **Library File Write** APIs for future subtitle, artwork,
      and sidecar writes.

## Artwork

- [ ] Add image proxy/cache routes with etag and variant support.
- [ ] Add thumbnail and preview-frame generation jobs.
- [ ] Add artwork candidate selection behavior.
- [ ] Add artwork export policy through **Library File Write** behavior.

## Search

- [ ] Upgrade SQLite fallback to FTS ranking/tokenization when bundled FTS
      support is guaranteed.
- [ ] Add item/person/tag/genre search filters.
- [ ] Add optional Tantivy or Meilisearch adapter boundary after SQLite FTS.

## Ingestion

- [x] Persist **Local Inference Evidence** for inferred kind, title, year,
      season, episode, confidence, evidence source, and inference version.
- [ ] Add rename/move detection using strong fingerprints when available.
- [ ] Use **Source Duplicate Relationship** when fingerprint evidence supports
      likely duplicate content.
