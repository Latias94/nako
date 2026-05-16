# Metadata-Catalog TODO

## M27.0 Domain Baseline

- [ ] Keep M27.0 design-only: no schema migrations, provider features, runtime
      behavior, or public API changes.
- [ ] Audit current **Media Item**, **Media Source**, catalog graph, metadata,
      NFO, artwork, and search surfaces against `CONTEXT.md`.
- [ ] Decide whether current `MediaKind` variants are enough for series,
      season, episode, collection, and extra-like items.
- [ ] Decide the first persistent source-to-item link model for multi-source
      items.
- [ ] Decide how to represent **Source Duplicate Relationship** separately
      from source identity and item merge behavior.
- [ ] Define **Metadata Source Priority** for local edits, NFO, TMDB, Douban,
      Bangumi, and future addon contributions.
- [ ] Define **NFO Round Trip** rules for import/export and unknown-field
      preservation.
- [ ] Define **Managed Artwork**, **Artwork Candidate**, and **Selected
      Artwork** storage/API expectations.
- [ ] Decide which server-foundation TODO items move into this workstream.
- [ ] Record ADRs for any schema, API, or provider-boundary decision that
      should not be rediscovered later.
- [ ] Produce an explicit handoff for M27.1/M27.2 implementation boundaries.

## Provider Breadth

- [ ] Add TMDB series, season, and episode support.
- [ ] Add Douban provider MVP.
- [ ] Add Bangumi provider MVP.
- [ ] Add item-level metadata profile overrides.

## NFO and Library File Writes

- [ ] Harden NFO round-trip behavior before broad export support.
- [ ] Decide how hard links and soft links are represented for non-local
      storage backends.
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

- [ ] Add rename/move detection using strong fingerprints when available.
- [ ] Use **Source Duplicate Relationship** when fingerprint evidence supports
      likely duplicate content.
