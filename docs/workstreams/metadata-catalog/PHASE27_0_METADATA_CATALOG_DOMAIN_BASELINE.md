# Phase 27.0: Metadata-Catalog Domain Baseline

## Status

Completed design baseline.

## Objective

Create an implementation-ready video-first media-server domain baseline for
Nako's metadata-catalog model before adding more provider breadth or
client-facing catalog behavior.

The current server can manage a movie-focused library, local metadata, TMDB
movie refresh, NFO import/export jobs, catalog graph browsing, and search
projection. The next risk is not a missing provider call; it is letting series,
anime, duplicate sources, NFO round trips, artwork, and addon contributions
grow through ad hoc fields.

## Domain Language

Use the terms defined in `CONTEXT.md`:

- **Media Library** is the configured collection boundary for storage,
  metadata, and permission context.
- **Media Domain** keeps the model open to video, audio, image, document,
  mixed, or online libraries without making M27 implement every domain.
- **Library Preset** is a user-facing configuration template, not item
  identity.
- **Media Source** is one discoverable playable file or remote object.
- **Source Locator** is library-scoped and must not become global identity.
- **Source Fingerprint** is evidence, not identity.
- **Source Duplicate Relationship** links likely duplicate sources without
  automatically merging them.
- **Media Item** is the user-facing catalog entry.
- **Episode-Like Item**, **Extra Item**, and **Franchise Collection** keep
  provider-specific watch-order and franchise relationships out of hard item
  identity.
- **Provider Subject** is provider-specific media evidence.
- **Provider Mapping** links provider evidence to Nako items without replacing
  Nako item identity.
- **Canonical Metadata** is the authoritative browsing/search/playback/export
  metadata.
- **Media Technical Facts** are observed source or stream facts.
- **Library Item State** and **User Playback State** are not canonical
  metadata.
- **Genre** and **Tag** are separate browsing concepts.
- **Review Rating**, **Content Rating**, and **User Rating** are separate
  rating concepts.
- **Metadata Source Priority** decides how local, NFO, provider, and future
  addon data fill fields.
- **NFO Round Trip** preserves content Nako does not own.
- **Library File Write** is mediated through Nako and must use VFS write/link
  capabilities instead of direct addon or provider path mutation.
- **Managed Artwork**, **Artwork Candidate**, and **Selected Artwork** are
  distinct concepts.

## Design Questions

M27.0 should answer these before implementation:

- Is `MediaKind` enough for movies, series, seasons, episodes,
  **Episode-Like Item**, **Extra Item**, **Franchise Collection**, and unknown
  video items, or does it need a larger hierarchy?
- How do **Media Domain** and **Library Preset** influence scan and metadata
  defaults without becoming hard item types?
- Does the existing `MediaItem.item_id` on `MediaSource` still work for
  multi-source items, alternate editions, and duplicate relationships?
- Which duplicate-source evidence is strong enough to create a **Source
  Duplicate Relationship**?
- Which fields are canonical, which are provider-specific, and which are local
  locks?
- How do TMDB, Douban, and Bangumi **Provider Subjects** map into ordinary
  Nako item kinds without hard-coding anime as a separate media kind?
- Which provider genres, tags, ratings, and content labels become canonical,
  and which remain provider evidence?
- What must NFO export preserve to satisfy **NFO Round Trip**?
- How should NFO, artwork, subtitle, and sidecar **Library File Writes** use
  VFS write/link capabilities for local and non-local storage?
- What should clients receive for artwork: provider URLs, managed image IDs,
  variant URLs, or selected artwork references?
- Which **Browse Facets** and **Sort Keys** are stable public contracts rather
  than raw database columns?
- Which decisions need an ADR before schema work starts?

## Current Code Audit

The current implementation already has useful video-first building blocks:

- `crates/nako-core/src/media.rs` defines `MediaDomain`, `LibraryPreset`,
  `LibraryOptions`, `MetadataProfile`, `MediaKind`, `MediaItem`,
  `MediaSource`, `CanonicalMetadata`, `MediaProbeResult`, graph records,
  `ImageAsset`, and artwork task records.
- `crates/nako-db/migrations/0001_initial.sql` and
  `0015_media_source_library_locator.sql` make source locator identity
  library-scoped.
- `crates/nako-db/migrations/0002_media_probe.sql` keeps source technical
  facts outside canonical item metadata.
- `crates/nako-db/migrations/0005_metadata_policy.sql` and
  `0016_metadata_provider_attempts.sql` provide field locks, raw provider
  response cache, and provider attempt diagnostics.
- `crates/nako-db/migrations/0006_library_profiles.sql` persists library
  domain, preset, and serialized library options.
- `crates/nako-db/migrations/0007_catalog_ingestion.sql` persists people,
  credits, genres, tags, collections, studios, image assets, search documents,
  scan state, and source fingerprints.
- `crates/nako-catalog/src/lib.rs` hydrates graph records and search
  projection from canonical metadata.
- `crates/nako-nfo/src/lib.rs` imports and exports a movie NFO subset through
  `nako-vfs::StorageBackend`.
- `crates/nako-search/src/lib.rs`, `crates/nako-db/src/search.rs`, and
  `crates/nako-server/src/http/catalog.rs` expose basic search through free
  text plus raw `facet` strings.

The important M27 gaps are:

- Provider identity is represented by `ExternalId` and raw response keys, but
  there is no durable **Provider Subject** or **Provider Mapping** model.
- `MediaSource.item_id` supports many sources per item, but there is no
  separate **Source Duplicate Relationship** table or relationship state.
- `MediaKind` has `Collection` and `Extra`, but no explicit distinction
  between **Episode-Like Item**, **Extra Item**, and **Franchise Collection**.
- `CanonicalMetadata.ratings` uses one `ContentRating` shape; provider review
  scores, age/content labels, and future user ratings are not separated.
- **Library Item State** and **User Playback State** are not yet modeled as
  durable first-class state.
- NFO export re-renders the owned movie subset; it does not preserve unknown
  XML when forced, so full **NFO Round Trip** is not yet satisfied.
- Artwork has image source, cache, and selected fields, but does not yet expose
  separate **Artwork Candidate**, **Selected Artwork**, and **Managed Artwork**
  client contracts.
- Browse and search accept raw facet strings and implicit ordering; there is
  no public **Browse Facet** or **Sort Key** enum/DTO contract.

## Baseline Decisions

### Item Hierarchy

M27 remains video-first. The target video hierarchy is:

- movie;
- series;
- season;
- episode;
- **Episode-Like Item** when provider evidence says a special, OVA, OAD, or
  similar entry belongs in watch order;
- **Extra Item** when the entry is related but outside the primary watch
  order;
- unknown video item when scan evidence is not enough.

Do not add an anime media kind. Anime is expressed by **Library Preset**,
provider priority, naming strategy, **Provider Mapping**, tags, and ordinary
movie/series/season/episode/extra item kinds.

Use **Franchise Collection** as a catalog relationship, not as source identity.
The current `MediaKind::Collection` should be treated as a compatibility
surface until M27.1 decides whether to keep it for collection landing pages or
move collection presentation entirely to the normalized collection graph.

### Media Domain And Library Preset

`Media Library` remains the management boundary. `MediaDomain` describes broad
processing capabilities. `LibraryPreset` only seeds editable defaults:

- scan and naming defaults;
- provider order;
- local metadata policy;
- image provider defaults;
- language and country defaults;
- presentation defaults.

Neither `MediaDomain` nor `LibraryPreset` may become item identity. A mixed
video library may contain movies, series, episodes, extras, and unknown video
items. An anime preset may produce ordinary movie or episode items.

### Source Links And Duplicates

Keep `MediaSource` identity library-scoped. A **Source Locator** is unique only
inside its **Media Library**. A **Source Fingerprint** is evidence, not source
identity.

For the first persistent source-to-item model, keep the existing
`MediaSource.item_id` primary link: multiple sources can point to one item.
Do not introduce many-to-many source membership unless M27.1 finds a concrete
edition, version, or alternate-cut workflow that needs it.

Add **Source Duplicate Relationship** as a separate future persistence model,
not as an item merge. A duplicate relationship should reference two source IDs,
record evidence such as strong fingerprint, size/etag, path evidence, or local
filesystem link evidence, and have a reviewable state such as suggested,
confirmed, or rejected. It must not automatically rewrite `MediaSource.item_id`
or collapse library context.

### Provider Mapping

Provider-specific records are **Provider Subjects**. TMDB movie/series/season
or episode IDs, Douban subjects, Bangumi subjects/episodes, IMDb IDs, and
future addon/provider subjects must map into Nako through **Provider Mapping**.

`ExternalId` may remain a compatibility field on canonical metadata, but it is
not enough for provider hierarchy, match confidence, provider subject type,
locale, or multiple subject candidates. M27.1 should add or refactor durable
provider mapping records before broad TMDB series, Douban, or Bangumi work.

Raw provider responses stay diagnostic and repeatability evidence. They are
not canonical metadata and do not replace provider mappings.

### Metadata Source Priority

Default metadata authority order is:

1. user-locked local edits;
2. NFO/local metadata when the library policy is local-first or read-only;
3. accepted addon or automation writes that pass through Nako-owned APIs and
   permissions;
4. built-in providers in the effective `MetadataProfile` order;
5. fallback provider data for fields still missing after higher-priority
   sources are applied.

Provider order is library/profile configuration, not a global constant. The
current presets may seed TMDB, Douban, or Bangumi defaults, but users should be
able to edit the effective metadata profile.

Provider data may fill unlocked fields. It must not overwrite user locks, and
it must not replace local/NFO-owned fields unless library policy explicitly
allows that behavior.

### Metadata, Facts, And State

Keep these layers separate:

- **Canonical Metadata** belongs to `MediaItem` and feeds browse, search,
  playback presentation, and NFO/artwork export.
- **Media Technical Facts** belong to `MediaSource` and streams, backed by
  probe records.
- **Library Item State** belongs to the library/item relationship, such as
  date added, visibility, or library-scoped presentation state.
- **User Playback State** belongs to a user/item or user/source relationship,
  such as progress, watched status, last played time, favorites, hidden status,
  and user rating.

Do not store playback progress, watched status, favorites, hidden state, last
played time, or user rating in `CanonicalMetadata`.

Split rating concepts before provider breadth expands:

- **Review Rating**: provider/community score with scale and vote count.
- **Content Rating**: age or content classification with region/source.
- **User Rating**: per-user rating in user state.

### NFO And Library File Writes

**NFO Import** stays enabled as local metadata input. **NFO Export** remains
opt-in through library policy.

M27.1/M27.2 must treat **NFO Round Trip** as stronger than the current movie
subset renderer. Unknown XML and third-party fields should survive when safe.
Nako-owned fields may be updated according to **Metadata Source Priority** and
field locks.

All NFO, artwork, subtitle, and sidecar writes are **Library File Writes**.
They must pass through Nako-owned APIs and VFS capability checks. Addons and
providers must not write library paths directly.

### Artwork

Provider URLs and local files are **Artwork Sources**. Client presentation
should use **Managed Artwork** references served by Nako, not remote provider
hotlinks.

M27.1/M27.2 should make these concepts explicit:

- **Artwork Candidate**: one discovered image choice with source, dimensions,
  language, provider/local origin, and cache state.
- **Selected Artwork**: the chosen candidate for a presentation slot such as
  poster, backdrop, logo, banner, thumbnail, or preview.
- **Managed Artwork**: Nako-owned cached or stored artwork with stable IDs,
  variant URLs, ETags, and cleanup policy.

The existing `ImageAsset` shape is a useful starting point, but its public API
contract should not expose provider hotlinks as the stable client contract.

### Browse Facets And Sort Keys

Client browse must use explicit names, not raw database columns or arbitrary
search strings.

Initial **Browse Facets** should be limited to supported dimensions:

- item kind;
- library;
- genre;
- tag;
- collection;
- studio;
- person/credit role;
- release year;
- content rating;
- provider mapping/provider ID when mappings exist.

Technical facets such as codec, resolution, audio language, and subtitle
language should wait until source technical facts are exposed through an
intentional browse contract.

Initial **Sort Keys** should be explicit names:

- title;
- sort title;
- release date;
- runtime;
- date added once **Library Item State** exists.

User-state sort keys such as recently played or user rating should wait until
**User Playback State** exists. Search ranking remains search-specific and is
not a generic sort key.

## Handoff To Implementation

M27.1 should be the schema and repository slice. It may touch:

- `nako-core` domain records and repository traits;
- `nako-db` migrations and adapters;
- compatibility reads for existing movie MVP data;
- focused repository tests.

M27.1 should not add provider API breadth or public client browse behavior
until the selected persistence model is covered.

After later domain review, M27.2 was narrowed to local inference and
provider-neutral provisional hierarchy before provider breadth. It may touch:

- scanner/name parser evidence, confidence, and version output;
- local inference persistence during scanning;
- provisional series/season/episode creation and unknown item fallback.

M27.3 should be the provider, NFO, and artwork expansion slice. It may touch:

- TMDB series/season/episode mapping;
- Douban and Bangumi provider mapping;
- NFO round-trip parsing/rendering and sidecar write policy;
- artwork candidate/selected/managed contracts;
- catalog hydration and search projection updates.

M27.2/M27.3 should not expose new browse/sort contracts until DTO names and
repository queries are explicit.

Client-facing browse/sort work should be a later M27 slice after M27.1 schema
decisions exist. It should introduce named **Browse Facets** and **Sort Keys**
in `nako-api` before routes accept them.

## ADR Disposition

ADR 0021 is accepted for M27.0. Additional ADRs are not required for this
design-only slice unless M27.1 changes schema, repository boundaries, or
public API DTOs in a way this phase note does not already cover.

## Implementation Sequence

1. Audit current code and docs against `CONTEXT.md`.
2. Write or update ADRs for item hierarchy, source relationships, metadata
   authority, and artwork ownership.
3. Move active metadata/catalog TODOs out of `server-foundation`.
4. Draft the schema/repository change plan for M27.1.
5. Implement local inference and provisional hierarchy in M27.2.
6. Only then implement provider breadth in M27.3.

## Non-Goals

- No provider API implementation in M27.0.
- No client UI.
- No automatic duplicate merge.
- No destructive NFO rewrite.
- No addon direct database or filesystem write path.

## Validation

Close-out gate for the planning slice:

- `git diff --check`
