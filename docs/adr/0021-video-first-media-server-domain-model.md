# 0021: Use a Video-First Media Server Domain Model

## Status

Accepted.

## Context

Nako is currently focused on the self-hosted video server backend: movies,
series, anime, home videos, playback, metadata, NFO, remote storage, and
transcoding. Long term, Nako should be able to grow toward a broader media
server scope similar to mature self-hosted servers, including music, podcasts,
photos, documents, mixed libraries, and online catalogs.

If the early model hard-codes video-only assumptions into libraries, metadata,
user state, browsing, search, and playback contracts, later media domains will
require broad migrations. If the early model tries to fully implement every
media domain now, the server foundation will become too broad before the video
experience is coherent.

## Decision

Nako will remain video-first in implementation scope while using a broader
media-server domain model.

`Media Library` is a management boundary with roots, scan policy, metadata
policy, permissions, and presentation defaults. A library has a `Media Domain`
such as video, audio, image, document, mixed, or online. A `Library Preset`
such as movies, TV, anime, music, podcast, photos, home video, mixed video, or
online catalog is a user-facing configuration template, not a hard permanent
content type.

`Media Item` remains the provider-neutral catalog entry. Movies, series,
seasons, episodes, collections, extras, and unknown items form the current
video-first hierarchy. Provider-specific concepts such as TMDB movies,
Bangumi subjects, Douban entries, anime specials, and future online catalog
subjects are mapped through provider mappings rather than replacing Nako item
identity.

Nako separates item information into three categories:

- canonical metadata: title, overview, dates, people, studios, countries,
  languages, genres, tags, review ratings, content ratings, images,
  collections, and external IDs;
- media technical facts: codec, container, resolution, bitrate, HDR, stream
  languages, subtitle languages, duration, file size, and source facts;
- user and library state: date added, watch state, playback progress,
  favorites, hidden state, last played time, and user rating.

Cross-domain or video-required fields may be modeled early as core metadata
fields. Domain-specific metadata for music, podcasts, photos, books, or online
catalogs should wait for the owning media-domain workstream unless a current
video-first feature requires it.

Client browsing should use explicit browse facets and sort keys rather than
assuming every database column is a stable public query contract.

## Consequences

- The video server can move quickly without closing the door on music,
  podcasts, photos, documents, mixed libraries, or online catalogs.
- Library presets can provide friendly setup defaults without becoming rigid
  item identity.
- Provider-specific concepts can be retained as mapping evidence without
  fragmenting Nako's catalog model.
- User playback state stays separate from canonical metadata, which avoids
  painful multi-user migrations later.
- Future media-domain workstreams can add domain-specific metadata tables or
  DTOs without overloading the video-first core.
- Some fields useful for future domains will remain intentionally absent until
  the relevant domain has real workflows and tests.

## Alternatives Considered

- Build a video-only model now: rejected because it would make future music,
  podcast, photo, and online-catalog support much harder to add cleanly.
- Implement every media domain immediately: rejected because it would dilute
  the current server focus before video playback, metadata, search, and
  clients are stable.
- Treat library preset as item identity: rejected because mixed libraries,
  anime movies, music videos, home videos, and online catalogs overlap across
  presets and domains.
- Expose raw database columns as client filters: rejected because browse and
  sort support should be a deliberate API contract.

## Related Workstreams

- `docs/adr/0010-library-presets-are-configuration-templates.md`
- `docs/adr/0011-normalized-catalog-graph-and-search-projection.md`
- `docs/workstreams/metadata-catalog/`
