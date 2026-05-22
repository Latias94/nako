# Phase 3.3: Library Profiles and Metadata Strategy

## Status

Implemented in the current workspace. This phase adds the first library
profile model and changes metadata refresh planning so provider choice comes
from the effective library profile instead of the HTTP endpoint.

## Problem

Nako currently has a minimal `Library` model and a TMDB movie refresh path.
That is enough for the first provider MVP, but it is not enough for a real
self-hosted media server.

Users need simple setup choices such as movies, TV, anime, music, podcasts,
photos, home videos, mixed video, and future online catalogs. These choices
should configure defaults, not become hard storage rules.

The core risk is mixing three different concepts:

- what processing pipeline the media needs
- what the item actually is
- what defaults the user wants for this library

## Proposed Model

### Library

`Library` is a management boundary:

- one or more roots
- scan options
- default naming strategy
- default metadata profile
- refresh schedule
- local metadata policy
- UI grouping and later permissions

The library should not be the only source of truth for item type.

### Media Domain

`MediaDomain` is a coarse capability class:

```text
video
audio
image
document
mixed
online
```

It decides broad processing capabilities such as probing, thumbnailing,
transcoding, embedded tag extraction, or catalog synchronization.

### Library Preset

`LibraryPreset` is a configuration template selected during setup:

```text
movies
tv
anime
music
podcast
photos
home_video
mixed_video
online_catalog
```

A preset populates editable defaults. For example:

- `movies`: TMDB first, optional Douban, movie filename parser
- `tv`: series/season/episode graph, episode naming parser
- `anime`: Bangumi/TMDB defaults, anime episode naming, local title tolerance
- `music`: embedded tags and MusicBrainz-style provider defaults later
- `podcast`: feed/RSS metadata defaults later
- `photos`: local metadata and thumbnail-first defaults
- `online_catalog`: addon/catalog provider defaults later

Presets must not prevent users from changing provider order or metadata policy.

### Media Kind

`MediaKind` remains item-level. Anime is not a core media kind. An anime movie
is still `movie`; an anime episode is still `episode`.

Future item kinds should be added when they describe a real item graph, such as
podcast episodes or music tracks, not merely a UI category.

### Metadata Profile

`MetadataProfile` describes metadata resolution behavior:

- applicable item kinds
- local metadata readers and order
- remote metadata providers and order
- image providers and order
- language and country
- refresh mode
- local metadata authority policy
- missing-only versus full-refresh behavior

This profile should be usable at library level first and item-kind level later.

## Refresh Modes

The first strategy model should support:

- `none`: do not refresh metadata automatically
- `validation_only`: validate existing IDs/local files without applying changes
- `default`: run configured providers using normal skip/cache rules
- `missing_only`: only fill empty unlocked fields
- `full_refresh`: re-fetch and replace unlocked fields

Field locks remain authoritative in every mode.

## Local Metadata Policy

Local metadata should be explicit:

- `disabled`: ignore local sidecar metadata
- `read_only`: import local metadata but do not write files
- `local_first`: local metadata wins and remote providers fill gaps
- `remote_first`: remote providers win unless fields are locked
- `write_sidecar`: export canonical metadata to NFO or sidecar files later

NFO remains a local metadata boundary and should not be mixed with remote
provider raw response cache.

## Provider Strategy

Provider selection should become data-driven:

```text
local_readers = [nfo]
metadata_providers = [tmdb, douban, bangumi]
image_providers = [tmdb]
```

The refresh job should resolve the effective strategy from:

1. item override, when present
2. item-kind profile inside the library
3. library default metadata profile
4. server fallback defaults

## Online Sources

Online video sites, Stremio-style catalogs, RSS feeds, and external catalog
services should be modeled as source or addon providers. They are not ordinary
local filesystem roots.

An `online_catalog` preset may exist later, but its root should be a provider
locator such as an addon URL or feed URL, not a local path.

## Deliverables

- [x] Add ADR 0010 for presets as configuration templates.
- [x] Extend the library design with domain, preset, options, and metadata profile
  concepts.
- [x] Persist library options in SQLite.
- [x] Define provider order and refresh mode types.
- [x] Change metadata refresh planning to use the effective library profile instead
  of hard-coding TMDB.
- [x] Keep TMDB as the first implemented remote provider.
- [x] Add tests for provider order, disabled providers, missing-only refresh, full
  refresh, and locked fields.

## Exit Criteria

- A library can store domain, preset, and metadata profile options.
- Preset defaults can be generated without locking the user into the preset.
- Metadata refresh resolves provider order from the library profile.
- A disabled provider is not called.
- `missing_only` does not overwrite populated unlocked fields.
- `full_refresh` updates unlocked fields while preserving locked fields.
- `cargo fmt`, `cargo check`, and `cargo nextest run` pass for the workspace.

## Out of Scope

- Full UI for editing profiles.
- Douban and Bangumi HTTP providers.
- MusicBrainz, podcast feed, or photo metadata implementations.
- Online catalog addon execution.
- Per-user library permissions.
- Item-level metadata profile overrides.
- Multi-provider fallback execution after the first provider fails or is not
  implemented.
