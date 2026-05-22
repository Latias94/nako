# 0010: Treat Library Presets as Configuration Templates

## Status

Accepted.

## Context

Nako needs a user-friendly way to create libraries for common use cases such as
movies, TV shows, anime, music, podcasts, photos, home videos, mixed video, and
future online catalogs. A direct mapping from these choices to hard library
types would be fragile:

- anime can be a movie, series, season, episode, OVA, special, or extra
- podcasts share audio capabilities with music but use different metadata
  sources and item graphs
- music videos are video playback items but often use music metadata semantics
- home videos and photos need local-first metadata and thumbnail behavior
- online catalogs are provider-backed sources, not ordinary local folders
- mixed libraries should remain possible without losing provider control

Jellyfin-style collection types, library options, metadata options, and provider
orders are useful reference concepts. Nako should keep the same separation of
concerns while avoiding a rigid content-type hierarchy too early.

## Decision

Nako will separate library management, media domain, library preset, item kind,
and metadata profile.

`Library` is a management boundary. It owns roots, scan behavior, default
metadata policy, refresh policy, and UI grouping. It must not be treated as the
source of truth for every item's final type.

`MediaDomain` describes broad processing capabilities:

- `video`
- `audio`
- `image`
- `document`
- `mixed`
- `online`

`LibraryPreset` is a user-facing configuration template. Presets may include:

- `movies`
- `tv`
- `anime`
- `music`
- `podcast`
- `photos`
- `home_video`
- `mixed_video`
- `online_catalog`

Choosing a preset should populate defaults such as naming strategy, provider
order, language, local metadata policy, image policy, and refresh mode. Users
must be able to edit the resulting options after creation.

`MediaKind` remains the item-level classification. For example, anime is not a
core media kind; anime movies are still movies and anime episodes are still
episodes.

`MetadataProfile` will define how metadata is resolved for a library or item
kind. It should include local readers, remote providers, image providers,
provider order, refresh mode, language, country, and local authority policy.

## Consequences

- The UI can offer simple choices without locking the storage model into those
  choices.
- Provider order can vary by library and item kind.
- A mixed video library can still contain both movies and series.
- Anime-specific behavior can be expressed as defaults for provider order and
  naming strategy, not as a separate permanent content type.
- Podcast and music can share audio infrastructure while using different item
  graphs and metadata providers.
- Online catalogs can be added later through source/addon abstractions instead
  of being forced into local filesystem scanning.

## Alternatives Considered

- Hard-code libraries by content type, such as movie, TV, anime, music, and
  photo. Rejected because boundaries overlap and would create migration
  pressure as soon as hybrid content appears.
- Use only a generic library with no presets. Rejected because first-run setup
  would be too manual and users expect reasonable defaults.
- Treat provider order as a global server setting only. Rejected because
  different libraries need different languages, sources, and local authority
  policies.

## Related Workstreams

- `docs/workstreams/server-foundation/PHASE3_3_LIBRARY_PROFILES_METADATA_STRATEGY.md`
- `docs/workstreams/server-foundation/TODO.md`
