# Taru Server Foundation Workstream

## Status

Draft. This workstream tracks the first server-side architecture decisions for
Taru before implementation expands beyond the initial binary crate.

Top-level tracking:

- [Goal map](../../GOALS.md)
- [Roadmap](../../ROADMAP.md)
- [Refactoring policy](../../development/REFACTORING_POLICY.md)

Current implementation focus is API, HTTP router, and SQLite repository
boundary cleanup ([Phase 23.0](PHASE23_0_API_HTTP_DB_BOUNDARY_CLEANUP.md))
after ingestion failure diagnostics. The local video-library playback MVP was
stabilized in [Phase 4.10](PHASE4_10_MVP_STABILIZATION.md).

## Goals

- Build Taru as a Rust workspace with clear server-side module boundaries.
- Keep `taru-server` focused on composition, application orchestration, and
  HTTP routing instead of letting giant integration test or service files
  become hidden architecture boundaries.
- Keep the first product shape as a modular monolith, not distributed services.
- Support local media libraries first while reserving a clean path for remote
  drives, WebDAV, S3-compatible stores, and future cloud drive connectors.
- Treat automation and AI as external-provider workflows instead of local model
  infrastructure in the early phases.
- Prefer language-agnostic extension protocols before in-process native plugins.

## Non-Goals

- No local vector database or local model runtime in the foundation phase.
- No early dependency on a separate search server, object store, or message bus.
- No direct loading of arbitrary native dynamic libraries as plugins.
- No OS-level FUSE mount as a core requirement for the first server milestone.

## Recommended Crate Boundaries

```text
crates/
  taru-server          # binary bootstrap, configuration, shutdown, composition
  taru-api             # HTTP API, OpenAPI, request/response DTOs
  taru-core            # domain models, IDs, errors, service traits
  taru-db              # migrations, repositories, transaction boundaries
  taru-library         # library scan, item graph, media source lifecycle
  taru-catalog         # catalog graph hydration and search projection updates
  taru-naming          # path and filename parsing
  taru-metadata        # TMDB/Douban/Bangumi providers and merge policy
  taru-nfo             # NFO import/export and local metadata source behavior
  taru-vfs             # internal virtual filesystem and storage backends
  taru-media-probe     # ffprobe/mediainfo abstraction and stream inspection
  taru-transcode       # FFmpeg plan builder and hardware acceleration policy
  taru-streaming       # direct play, remux, transcode, HLS/session decisions
  taru-search          # embedded and optional external search adapters
  taru-events          # domain events, outbox, webhook delivery
  taru-automation      # external AI/API provider automation workflows
  taru-addon-protocol  # HTTP addon manifest and resource contract
```

The initial binary can still be small. The important part is to keep domain
boundaries explicit before cross-cutting concerns such as remote storage,
transcoding, and extension hooks become entangled.

## Foundation Constraints

- Use bounded async pipelines for scan, probe, metadata, webhook, automation,
  and transcode work.
- Model expensive work with explicit resource classes and conservative defaults.
- Keep batch work idempotent so retries and repeated scans are safe.
- Isolate per-item failures unless a strict all-or-nothing mode is requested.
- Prefer persisted job inputs and recoverable progress over in-memory-only
  orchestration state.

## Research Summary

### Automation and AI

AI should start as a workflow capability:

- Provider credentials are user-managed secrets.
- Jobs are explicit, auditable, cancellable, and rate-limited.
- Providers may call OpenAI-compatible gateways, recommendation services, or
  custom HTTP endpoints.
- Results are stored as suggestions or generated artifacts, not as hidden
  mutations of canonical metadata.

This keeps edge-case experience improvements possible without committing to
local model serving, vector search, GPU scheduling, or model lifecycle work.

### Async and Concurrency

Taru should use async Rust for I/O-heavy orchestration, but async fan-out must
always be bounded. Pipelines such as media probing, metadata refresh, webhook
delivery, automation calls, and future transcode queues should define their
own concurrency limits and resource classes. This keeps large libraries,
remote drives, and weaker self-hosted machines stable under load.

### Internal Virtual Filesystem

Taru should implement an internal VFS abstraction before considering OS-level
mounting. The scanner, metadata resolver, probe layer, and transcode layer
should consume `taru-vfs` instead of `std::fs` directly.

Key capabilities:

- `list`, `stat`, `open`, `open_range`, `etag`, `fingerprint`
- capability flags such as seekable, range-readable, watchable, linkable,
  writable, expensive-listing, rate-limited, and remote-latency
- separate directory metadata cache and byte-range media cache
- path normalization and traversal protection
- provider-aware concurrency limits and retry policy

OS-level FUSE, WebDAV export, or rclone mounts can be supported later as
integration points. They should not define Taru's core storage model.

### Stremio-Style Addons

Stremio's addon model is useful because extensions are HTTP services described
by a manifest and filtered by declared resource capabilities. Taru can adopt the
same principle without adopting the Stremio protocol wholesale.

Recommended direction:

- Define a Taru addon manifest with resources such as metadata, image, stream,
  subtitle, automation, webhook, and catalog.
- Let addons run out-of-process over HTTP first.
- Provide a JavaScript/TypeScript SDK later as developer ergonomics, not as the
  server's plugin runtime.
- Consider a Stremio-compatible export addon so Taru libraries can be exposed
  to Stremio clients as a separate compatibility feature.

### Search

The search boundary should be an internal trait. Embedded search can start with
SQLite FTS or Tantivy. Meilisearch should be an optional adapter for users who
want a dedicated search service.

Search should index the catalog graph rather than only raw item titles. People,
roles, tags, genres, collections, studios, file names, external IDs, and
localized aliases should become searchable through a denormalized index fed by
catalog events.

### Catalog Graph and Artwork

Taru needs a normalized catalog graph before it grows more provider integrations:

- people and item credits for actor/director/writer pages;
- user tags separate from provider genres;
- collections as first-class entities, not only labels;
- studios and production companies as graph nodes;
- image assets with owners, variants, cache state, and provider provenance.

Artwork should be served through an explicit cache/proxy pipeline. List routes
should return image references, not bytes. Poster grids, person images,
backdrops, and preview frames must be lazy-loaded by clients and generated or
downloaded through bounded background jobs.

### Library Profiles and Presets

Libraries should be management boundaries, not hard content-type boundaries.
A library owns roots, scan rules, default naming strategy, metadata profile,
refresh policy, local metadata policy, and later UI grouping or permissions.

Taru should separate:

- `MediaDomain`: broad processing capability such as video, audio, image,
  document, mixed, or online
- `LibraryPreset`: a user-facing template such as movies, TV, anime, music,
  podcast, photos, home video, mixed video, or online catalog
- `MediaKind`: the item-level graph type such as movie, series, season,
  episode, collection, extra, or future audio/photo kinds
- `MetadataProfile`: provider order, local readers, image providers, language,
  country, refresh mode, and local authority policy

Presets should populate editable defaults. For example, choosing anime may set
Bangumi/TMDB defaults and anime naming rules, but it must not make anime a hard
media kind. An anime movie is still a movie, and an anime episode is still an
episode.

### Remote Storage

Remote storage should be modeled as source backends, not as ordinary local
folders. For example:

- `local://`
- `webdav://`
- `s3://`
- `http+range://`
- `rclone://`

For object and WebDAV-like stores, Apache OpenDAL and Apache Arrow
`object_store` are strong Rust candidates. They are better architectural fits
than directly coupling Taru to a specific S3-compatible server.

## Reference Links

- Stremio addon basics: https://stremio.github.io/stremio-addon-guide/basics
- Stremio addon resources: https://stremio.github.io/stremio-addon-sdk/api/
- Stremio manifest format: https://stremio.github.io/stremio-addon-sdk/api/responses/manifest.html
- rclone mount and VFS cache: https://rclone.org/commands/rclone_mount/
- Apache OpenDAL services: https://opendal.apache.org/core/
- Apache Arrow object_store: https://docs.rs/object_store/latest/object_store/
