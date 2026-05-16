# Metadata-Catalog Milestones

## M27.0: Metadata-Catalog Domain Baseline

Status: completed.

Outcome: Taru has a documented, implementation-ready video-first media-server
domain model for growing from movie-first metadata into series, episode,
anime, **Episode-Like Item**, **Extra Item**, **Franchise Collection**,
artwork, duplicate-source, NFO-safe catalog behavior, and explicit client
browse contracts.

Scope: design baseline only. M27.0 should not add schema migrations, provider
features, runtime behavior, or public API changes. It should make the next
implementation slices unambiguous.

Deliverables:

- Audit current `taru-core`, `taru-db`, `taru-catalog`, `taru-metadata`,
  `taru-nfo`, `taru-vfs`, and HTTP DTO surfaces against `CONTEXT.md`.
- Decide the first stable **Media Item** hierarchy for movie, series, season,
  episode, **Episode-Like Item**, **Extra Item**, **Franchise Collection**,
  and unknown video items.
- Decide how **Media Domain** and **Library Preset** affect scanning,
  metadata defaults, and presentation without becoming item identity.
- Define **Provider Subject** and **Provider Mapping** rules for TMDB, Douban,
  Bangumi, and future provider/addon evidence.
- Decide how **Media Source** links to **Media Item** when one item has
  multiple playable sources.
- Decide the persistence shape for **Source Duplicate Relationship** without
  automatic source or item merging.
- Define **Metadata Source Priority** and field-lock behavior for local edits,
  NFO, TMDB, Douban, Bangumi, and future addon contributions.
- Separate **Canonical Metadata**, **Media Technical Facts**, **Library Item
  State**, and **User Playback State** in the target model.
- Separate **Genre** from **Tag**, and **Review Rating**, **Content Rating**,
  and **User Rating** from each other.
- Define **NFO Round Trip** requirements for import/export without destructive
  rewrites, including how sidecar writes consume VFS write/link capabilities.
- Define the first **Managed Artwork**, **Artwork Candidate**, and **Selected
  Artwork** route/storage contract.
- Define the initial **Browse Facets** and **Sort Keys** that clients may rely
  on.
- Move relevant TODO items out of `server-foundation` into this workstream.
- Record any architecture decision that materially changes schema, provider
  boundaries, or public API DTO shape.

Exit criteria:

- `CONTEXT.md` uses stable terms for the selected model
- ADR 0021 is accepted
- M27.1 and M27.2 implementation boundaries are explicit
- current code gaps are recorded with enough detail for implementation
- required ADRs are created or linked
- `git diff --check`

Close-out evidence:

- [Phase 27.0](PHASE27_0_METADATA_CATALOG_DOMAIN_BASELINE.md) records the
  code audit, baseline decisions, and handoff to M27.1/M27.2.
- [ADR 0021](../../adr/0021-video-first-media-server-domain-model.md) is
  accepted.
- [TODO](TODO.md) marks the M27.0 design-baseline checklist complete.
- `git diff --check` passed with Git CRLF normalization warnings only.

## M27.1: Catalog Schema and Repository Slice

Status: completed.

Outcome: the database and repository seams can persist the selected
metadata-catalog model.

Deliverables:

- Add or refactor durable records for selected item hierarchy and source links.
- Add migration coverage for duplicate-source relationships if selected in
  M27.0.
- Add focused repository tests before changing provider behavior.
- Keep old movie MVP behavior working through compatibility queries or a
  deliberate migration plan.

Exit criteria:

- `cargo fmt --all -- --check`
- `cargo check --workspace --tests`
- focused `cargo nextest run -p taru-db`
- focused `cargo nextest run -p taru-core`
- `git diff --check`

Close-out evidence:

- [Phase 27.1](PHASE27_1_CATALOG_SCHEMA_REPOSITORY_SLICE.md) records the
  implemented schema/repository slice and remaining M27.2 boundaries.
- `crates/taru-core` owns provider mapping, duplicate-source, and local
  inference evidence records and repository traits.
- `crates/taru-db/migrations/0018_metadata_catalog_domain.sql` persists the
  selected M27.1 model.
- `cargo nextest run -p taru-db` passed with 31 tests.
- `cargo nextest run -p taru-core` passed with 3 tests.
- `cargo fmt --all -- --check`, `cargo check --workspace --tests`, and
  `git diff --check` passed.

## M27.2: Local Inference and Provisional Hierarchy Slice

Status: completed.

Outcome: scanning can persist local inference evidence and create a
provider-neutral provisional video hierarchy before provider or NFO
confirmation.

Deliverables:

- Extend `taru-naming` parsed-name output with confidence, evidence source,
  parser version, and inferred kind/title/year/season/episode fields.
- Persist **Local Inference Evidence** from `taru-library` scanning.
- Create provisional series, season, episode, and unknown item hierarchy during
  scanning.
- Keep provider/NFO confirmation as a later upgrade path.

Non-goals:

- No TMDB, Douban, or Bangumi breadth.
- No NFO confirmation.
- No Source Variant UI.
- No browse API.
- No rating or user-state work.

Exit criteria:

- `cargo fmt --all -- --check`
- `cargo check --workspace --tests`
- focused `cargo nextest run -p taru-naming`
- focused `cargo nextest run -p taru-library`
- focused `cargo nextest run -p taru-db`
- `git diff --check`

Close-out evidence:

- [Phase 27.2](PHASE27_2_LOCAL_INFERENCE_PROVISIONAL_HIERARCHY.md) records the
  scan-path local inference implementation and remaining M27.3 boundaries.
- `taru-naming` emits confidence, evidence source, evidence value, parser
  version, and unknown fallback results.
- `taru-library` writes source-owned **Local Inference Evidence** and creates
  provisional series/season/episode items during indexing.
- `taru-db/migrations/0019_library_item_states.sql` persists library-scoped
  source-less item membership.
- `taru-db/migrations/0020_local_inference_evidence_snapshot_key.sql` keeps
  local inference evidence as a source-owned current snapshot rather than an
  append-only scan log.
- Confirmed items are protected from local-inference canonical metadata
  overwrites during rescan.
- `cargo nextest run -p taru-naming` passed with 6 tests.
- `cargo nextest run -p taru-library` passed with 15 tests.
- `cargo nextest run -p taru-db` passed with 32 tests.
- `cargo fmt --all -- --check`, `cargo check --workspace --tests`, and
  `git diff --check` passed.

## M27.3: Provider and NFO Expansion Slice

Status: completed.

Outcome: built-in providers and NFO import/export can populate the expanded
catalog model without bypassing local authority rules.

Deliverables:

- Add a shared **Hierarchy Confirmation** service boundary for provider/NFO
  confirmation of provisional hierarchy.
- Confirm provisional series, season, and episode items in place by updating
  existing `MediaItem` rows and clearing `LibraryItemState.provisional`.
- Write accepted **Provider Mapping** records when provider metadata refresh
  succeeds.
- Add TMDB series, season, and episode mapping.
- Add Douban provider MVP when the domain model can store its identifiers and
  localized metadata safely.
- Add Bangumi provider MVP when anime profiles can map to ordinary movie or
  episode item kinds instead of a separate hard media kind.
- Preserve local edits and NFO authority through **Metadata Source Priority**.
- Keep NFO unknown-content round-trip preservation as a separate **NFO Round
  Trip** hardening follow-up.

Non-goals:

- No Source Variant UI.
- No browse API.
- No artwork candidate, selected artwork, or managed artwork expansion.
- No rating, user state, or browse facet work.
- No automatic duplicate merge.
- No general **Hierarchy Repair** flow.

Exit criteria:

- `cargo fmt --all -- --check`
- `cargo check --workspace --tests`
- focused `cargo nextest run -p taru-db`
- focused `cargo nextest run -p taru-library`
- focused `cargo nextest run -p taru-metadata`
- focused `cargo nextest run -p taru-nfo`
- focused provider and NFO nextest runs
- `git diff --check`

Close-out evidence:

- [Phase 27.3](PHASE27_3_HIERARCHY_CONFIRMATION_PROVIDER_NFO.md) records the
  hierarchy confirmation, provider mapping, TMDB series/season/episode, and
  NFO episode confirmation slice.
- `taru-metadata` owns the shared **Hierarchy Confirmation** service boundary.
- Metadata refresh writes accepted **Provider Subject** and **Provider
  Mapping** records for successful TMDB, Douban, and Bangumi fetches.
- `taru-nfo` confirms provisional episode hierarchy in place through the
  shared service.
- `cargo nextest run -p taru-db --no-fail-fast` passed with 32 tests.
- `cargo nextest run -p taru-library --no-fail-fast` passed with 15 tests.
- `cargo nextest run -p taru-metadata --no-fail-fast` passed with 26 tests.
- `cargo nextest run -p taru-nfo --no-fail-fast` passed with 8 tests.
- `cargo fmt --all -- --check`, `cargo check --workspace --tests`, and
  `git diff --check` passed.
