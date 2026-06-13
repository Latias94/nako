# Jellyfin Reference Architecture Notes

## Scope And License Boundary

This note records architecture and product-capability observations from the local
Jellyfin reference checkout under `repo-ref/jellyfin`. It is intentionally
structural: no Jellyfin source, comments, tests, migrations, schemas, assets, or
generated artifacts are copied, translated, or rewritten for Nako.

The useful reference value is not implementation detail. It is the shape of a
mature self-hosted media server: which domains became first-class, where stable
interfaces exist, and which operational surfaces users eventually expect.

## Executive Summary

Jellyfin is organized around a broad media-server product surface, not only a
video playback engine. The solution separates host startup, HTTP API,
client-facing models, controller/service interfaces, implementation-heavy
runtime modules, metadata providers, local metadata/NFO handling, media
encoding, database entities, and test projects. The names have historical Emby
inheritance, but the shape is clear: mature media servers accumulate depth in
library scan/index, metadata/provider orchestration, playback/transcode,
session/user/device policy, plugin/package handling, scheduled operations,
backup/migration, and client contracts.

For Nako, the main lesson is not to copy Jellyfin's in-process plugin or class
hierarchy model. Nako already has a stronger Rust modular-monolith direction:
typed playback planning, Addon sidecars, VFS boundaries, provider-neutral
metadata review, durable jobs, protocol-owned public DTOs, and redaction-first
Admin APIs. The gap is product depth and contract breadth: device profiles,
large-library watcher/scheduler integration, rich user/session/device policy,
realtime client updates, backup/restore execution, broader media-domain support,
artwork/trickplay/media-segment maturity, and operator-facing configuration and
diagnostics.

## Core Domain Model And Module Shape

Jellyfin's reference tree shows a layered domain:

- `MediaBrowser.Controller` acts as a service/interface and internal domain
  contract layer. It has domain entities for videos, movies, series, seasons,
  episodes, audio, music albums/artists/genres, books, photos, collections,
  people, studios, years, folders, user views, user item data, linked children,
  media sources, and query objects.
- `MediaBrowser.Model` acts as the client/API contract layer. It contains DTOs,
  query models, media info records, session commands, user policy records, DLNA
  device profiles, scheduled task models, plugin/package models, system info,
  updates, subtitles, search, and configuration.
- `Emby.Server.Implementations` is the broad runtime implementation layer:
  application host, library manager, file monitor, resolvers, search, user data,
  scheduled tasks, plugins, updates, session/WebSocket, images, localization,
  configuration, quick connect, sync play, playlists, collections, and IO.
- `Jellyfin.Server.Implementations` carries newer implementation slices around
  activity, devices, event consumers, item repositories, backup, media segments,
  security, trickplay, users, and database configuration.
- `Jellyfin.Api` is a wide HTTP boundary with many feature-specific controllers,
  API models, auth policies, middleware, streaming helpers, and WebSocket
  listeners.
- `MediaBrowser.Providers`, `MediaBrowser.LocalMetadata`, and
  `MediaBrowser.XbmcMetadata` separate remote providers, local image/XML
  metadata, and NFO/XBMC-compatible import/export concerns.
- `MediaBrowser.MediaEncoding` plus `src/Jellyfin.MediaEncoding.Hls` and
  `src/Jellyfin.MediaEncoding.Keyframes` separate FFmpeg/probe/transcode,
  subtitle encoding, HLS playlist/keyframe work, and media technical extraction.
- `Jellyfin.Data` and `src/Jellyfin.Database` expose database-facing entities,
  enums, query helpers, provider-specific database adapters, and migrations.

Nako already has a cleaner Rust vocabulary for many of these ideas:
`nako-core`, `nako-library`, `nako-metadata`, `nako-catalog`, `nako-search`,
`nako-playback`, `nako-transcode`, `nako-streaming`, `nako-vfs`, `nako-api`,
`nako-client-protocol`, `nako-server`, and the addon crates. The durable lesson
is to keep deepening these modules by workflow and interface, instead of
collapsing mature product behavior into one server implementation crate.

## Capability Surface Observations

### Library Scan, Index, And Browse

Jellyfin treats library intake as a full product subsystem:

- naming/parsing has dedicated modules for audio, audiobooks, books, TV,
  external files, and video;
- library resolvers exist per media family and container shape;
- IO includes library monitoring and file-refresh behavior;
- scan/index flows have post-scan validators for people, genres, studios,
  artists, music genres, collections, and splash screens;
- library queries, search hints, user views, item counts, linked children,
  collections, similar items, next-up, chapters, media streams, keyframes, and
  media attachments are separate concerns;
- library structure APIs expose virtual folders, media paths, library options,
  and library update information.

Nako already has durable scan state, source tombstones, source fingerprinting,
local inference, provisional hierarchy, scan-originated source-hash triggering,
VFS cache repair, and source duplicate reconciliation. The remaining Jellyfin
class of maturity is less about "can scan files" and more about:

- productized watcher/debounce integration for large copies and remote storage;
- scan scheduling and queue pressure visible to the operator;
- post-scan derived-state maintenance for people, genres, collections,
  similar/next-up, search facets, trickplay, chapters, lyrics, subtitles, and
  artwork;
- library-type presets that enable broad media domains without changing source
  identity;
- user-facing repair flows for stale, moved, duplicate, incomplete, and
  unsupported media.

### Metadata, Providers, NFO, Images, Subtitles, Lyrics

Jellyfin's metadata area is broad and split by provider role:

- provider interfaces distinguish local metadata, remote metadata, remote
  search, remote image, local image, dynamic image, external IDs, external URLs,
  pre-refresh hooks, change monitors, and provider managers;
- metadata service files exist per media type: movies, TV series/seasons/
  episodes, music albums/artists/audio, books/audiobooks, photos, playlists,
  genres, studios, people, live TV, channels, trailers, and years;
- provider plugins include TMDB, OMDb, MusicBrainz, AudioDB, ListenBrainz,
  StudioImages, and book-related ID/URL providers;
- local metadata and NFO handling are separate from remote providers;
- media-info providers cover ffprobe-backed facts, embedded images, subtitles,
  audio images, video images, and lyrics;
- scheduled tasks exist for subtitle and lyric workflows.

Nako has a strong provider-neutral metadata model: Provider Subjects, Provider
Mappings, Candidate Graphs, durable Candidate Review, accepted-review
application, generated artifact apply, and Admin/Web governance. Nako also
has NFO import/export and managed artwork foundations. The gap is breadth and
operator ergonomics:

- richer provider registry/status/configuration and per-library provider
  priority controls;
- provider-specific diagnostics that are useful but do not leak raw payloads,
  headers, proxy URLs, paths, tokens, or provider bodies;
- metadata undo/audit for applied provider mappings and hierarchy repair;
- broader built-in provider coverage for music, books, photos, comics, lyrics,
  subtitles, and external identifiers;
- artwork derivative lifecycle: size presets, modern formats, placeholder
  hashes, invalidation, and cache-policy ownership;
- sidecar/local authority workflows that make NFO Round Trip, local images,
  subtitles, and lyrics understandable to operators.

### Playback, Transcode, Streaming, And Media Technical Facts

Jellyfin's playback surface spans DTOs, planner-like models, FFmpeg adapters,
controllers, HLS helpers, keyframe extraction, subtitles, attachments, media
segments, DLNA/device profiles, and session telemetry:

- client capability DTOs and session models include playback start/progress/
  stop, player state, play queues, repeat/shuffle, transcode reasons, and
  transcoding info;
- DLNA models include device profiles, direct play profiles, codec/container/
  subtitle/transcoding profiles, resolution logic, stream builders, and
  playback error codes;
- media encoding modules separate FFmpeg validation, probing normalization,
  stream/chapter/side-data facts, subtitle parsing/encoding, attachment
  extraction, transcode management, HLS playlist generation, keyframe
  extraction, and media segment management;
- API controllers expose dynamic HLS, HLS segments, universal audio, media
  info, videos/audio, video attachments, trickplay, subtitles, playstate, and
  sessions.

Nako is already strong in typed playback planning: Direct Play first, Remux
before Transcode, playback capability profiles, HLS/fMP4/adaptive ladder
foundations, subtitle sidecar/burn-in planning, audio compatibility, HDR tone
mapping, FFmpeg command planning, resource admission, playback tickets, renderer
transport tickets, and manifest-backed artifacts. Jellyfin still highlights
several maturity gaps:

- a richer device-profile database and client capability reporting surface;
- precise compatibility reasons suitable for client UI and support diagnostics;
- active session limits and per-user playback policy controls;
- bandwidth-aware ABR and variant pruning;
- seek/restart polish, keyframe discipline, and eventual LL-HLS/CMAF decisions;
- trickplay generation and media segment APIs as user-visible playback
  navigation features;
- offline sync/download artifacts separate from temporary playback transcodes;
- optional remote transcode workers or distributed execution only after local
  runtime invariants are stable.

### Users, Sessions, Devices, Auth, And Access

Jellyfin exposes user, device, session, authentication, authorization, quick
connect, display preferences, playstate, SyncPlay, and WebSocket surfaces as
first-class product features:

- server-side auth has provider and password-reset extension points;
- API auth policies cover anonymous LAN access, first-time setup, elevated
  local access, user permissions, default authorization, and SyncPlay access;
- session runtime includes WebSocket management and session listeners;
- models include user policy, device info, display preferences, quick connect
  DTOs, session commands, playback commands, and user item data;
- database entities include users, permissions, preferences, devices, API keys,
  access schedules, user data, and display preferences.

Nako's Single-Admin Mode is a good first slice, but should remain explicitly
temporary. Mature gaps to consider:

- multi-user account lifecycle, roles, and library access beyond one operator;
- device registration, device policy, and API key/session inventory;
- local network versus remote access policy;
- account recovery and password reset;
- per-user playback cost policy, remote bitrate, transcode permission, and
  active-session limits;
- playback-state conflict semantics across devices;
- realtime session and playback updates for clients;
- shared watch/SyncPlay-like behavior only after session primitives are
  reliable.

### Plugin And Extensibility Surface

Jellyfin has a classic in-process plugin/package surface:

- plugin manager and plugin load context;
- plugin model DTOs, plugin pages, plugin status, package/update models;
- package and plugin API controllers;
- plugin update scheduled task;
- provider plugins that embed provider capability, configuration, images, and
  web configuration pages;
- event consumers for plugin install/update/uninstall notifications.

Nako should not use this as a compatibility target. Nako's Addon vocabulary is
better aligned with the desired trust model: Addon Sidecars, Addon Protocol,
Addon Tokens, scoped grants, Addon Tasks, Addon Event Subscriptions, Hosted
Pages, Configuration Schemas, Nako-Managed Artifacts, and Library File Write
APIs.

The Jellyfin lesson is the user-facing product shape:

- users expect discovery, install/update status, configuration pages, health,
  scheduled work, and error reporting;
- providers and extensions need lifecycle visibility, not only a callout hook;
- extension-provided UI should be surfaced predictably;
- package repositories and update checks become an operational concern.

For Nako, the safer architecture path is:

- keep Addon execution out of process;
- add official catalog/package descriptors and install guides before process
  supervision;
- add health checks and token rotation before marketplace-like automation;
- route addon writes through host-owned APIs and audit;
- use Addon Suite for deployment convenience while retaining per-Addon grants.

### Admin, Operations, Release, And Maintenance

Jellyfin's tree shows many operator-facing surfaces:

- system/startup/dashboard/environment/configuration/backup controllers;
- scheduled tasks for refresh, cleanup, database optimization, cache cleanup,
  log cleanup, transcode cleanup, plugin updates, chapter images, media
  segments, audio normalization, and people validation;
- migrations and pre-startup routines;
- full-system backup service;
- activity/event logging and notification consumers;
- health checks, OpenAPI/Redoc/Swagger assets, custom schema filters, and API
  documentation hosting;
- deployment templates and self-hosting assets;
- localization and ratings data.

Nako already has release gates, M1 ladder evidence, self-hosted docs, backup/
restore docs, Admin diagnostics, durable jobs, runtime budgets, HTTP trace IDs,
VFS repair, and PostgreSQL/SQLite boundaries. Remaining mature-server gaps:

- executable backup/restore flows and backup classification for each artifact
  class;
- config mutation authority: hot-apply versus restart-required settings;
- richer Admin job drilldowns, retries, cancellation, and incident bundles;
- safe realtime diagnostics for scan/transcode/session changes;
- explicit reverse-proxy, HTTPS, DDNS, LAN/remote endpoint and tunnel cookbook;
- container hardware pass-through evidence and optional one-frame GPU smoke;
- API scale and cache contracts across catalog, images, search, playback, and
  Admin pages;
- localization and rating-system data strategy if Nako targets a broader user
  base.

### Client Contract

Jellyfin's API and model layers show the value of a broad, stable client
contract:

- controllers are feature-specific and numerous rather than one generic server
  API;
- models cover DTOs, queries, sessions, DLNA/device profiles, tasks, updates,
  plugin information, users, system info, media info, subtitles, search,
  playlists, live TV, and configuration;
- integration tests cover controllers, OpenAPI, auth helpers, WebSocket, and
  plugin surfaces.

Nako has already made a good long-term choice by keeping `nako-client-protocol`
permissive and dependency-light, with `nako-api` as the AGPL mapping/adapter
layer and `nako-client`/SDKs as consumers. The Jellyfin comparison suggests
future pressure in these areas:

- public route breadth for browse/search/item detail/playback/session state;
- stable pagination and cache validators for large libraries;
- generated SDK parity and route inventory checks;
- client capability reporting and device profiles;
- realtime gateway payloads that are principal- and library-filtered;
- a Media Web client surface that proves the API is actually usable, not only
  well typed.

## Mature Capabilities Nako Should Consider

Near-term, aligned with M1-M3:

- watcher/debounce productization and scan scheduler integration;
- device capability reporting and profile-driven playback reasons;
- Admin-visible playback/session limits and per-user playback policy;
- richer Media Web browse/player smoke coverage;
- trickplay/keyframe/media-segment planning as a playback UX feature;
- backup/restore execution, not only docs;
- safe client realtime gateway for scan, catalog, playback, and job updates.

Medium-term, aligned with M4-M5:

- metadata undo/audit and hierarchy repair workflows;
- richer provider diagnostics, configuration, and provider priority UI;
- artwork derivative cache and placeholder strategy;
- broader built-in metadata domains: music, books, photos, comics, lyrics,
  subtitles, and external identifiers;
- Addon catalog/package/install-guide/health/token-rotation flows;
- hosted addon settings and diagnostics surfaces.

Longer-term product breadth:

- live TV/tuner/EPG/recording only if Nako chooses that product scope;
- DLNA/casting/device discovery if target clients require it;
- offline sync/download artifacts with quotas, expiry, revocation, and resumable
  transfer;
- SyncPlay/shared-watch behavior after realtime/session foundations mature;
- localization and regional rating systems.

## Nako Architecture Lessons In Module / Interface / Seam / Adapter / Depth / Leverage / Locality Terms

### Module

Jellyfin's mature product shape validates Nako's crate separation. Nako should
continue keeping domain records in `nako-core`, persistence in `nako-db`,
provider runtime in `nako-metadata`, scan/intake in `nako-library`, playback
decisions in `nako-playback`, FFmpeg planning/runtime in `nako-transcode`,
byte transport in `nako-streaming`, public contracts in
`nako-client-protocol`, and composition/HTTP in `nako-server`. Add new crates
only when multiple real callers need the boundary; otherwise deepen existing
modules.

### Interface

Jellyfin has many interfaces for item resolution, metadata providers, image
providers, media source providers, session management, user management,
scheduled tasks, device management, live TV, transcode management, and
repositories. Nako should keep using workflow-shaped ports rather than broad
mechanical traits. Existing examples such as `CatalogHydrationPort` and
`MetadataRefreshPort` are the right direction. Good future interface candidates
are device capability profiles, scan scheduler/work admission, trickplay
generation, backup execution, realtime event gateway, and Addon health/install
guide surfaces.

### Seam

The high-value seams in Jellyfin are provider, resolver, media source, scheduled
task, plugin, transcode, device profile, and API contract seams. Nako should
map these into its own language:

- resolvers become Local Inference plus Hierarchy Confirmation, not provider
  identity;
- providers emit Provider Subjects, Provider Mappings, Candidate Graphs, and
  Artwork Candidates;
- plugin hooks become Addon Resources, Addon Tasks, Addon Event Subscriptions,
  and Addon Hosted Pages;
- playback decisions remain Playback Runtime owned;
- scheduled work becomes durable jobs with resource classes and redacted
  diagnostics.

### Adapter

Jellyfin has adapters for FFmpeg/ffprobe, metadata providers, NFO/local files,
SQLite/EF, plugin packages, DLNA, live TV tuner hosts, WebSocket, and HTTP API
models. Nako should keep adapters typed and redaction-safe: provider HTTP
clients, VFS backends, DB backends, FFmpeg runners, Addon sidecars, renderer/
casting adapters, and future realtime transports should not leak local paths,
raw provider payloads, tokens, command lines, or filesystem identity across API
boundaries.

### Depth

Mature systems have deep modules for small-looking features: subtitles, lyrics,
trickplay, keyframes, media segments, image derivatives, display preferences,
scheduled cleanup, user data, device profiles, and migrations. Nako should
prefer deepening one workflow until it has domain records, repository contract,
runtime policy, Admin/Public DTOs, tests, diagnostics, and docs before starting
many shallow features.

### Leverage

Highest leverage gaps for Nako, based on Jellyfin comparison:

1. Device capability and playback compatibility profiles, because they improve
   every client and explain Direct/Remux/Transcode choices.
2. Watcher/scheduler/queue-pressure maturity, because large-library reliability
   is a core self-hosted expectation.
3. Multi-user/session/device policy, because even a single-admin first release
   should not hard-code single-user assumptions.
4. Backup/restore and incident diagnostics, because self-hosted operators need
   trust before adding more automation.
5. Metadata undo/repair and artwork/trickplay depth, because catalog quality is
   the visible product.
6. Addon catalog/health/configuration, because extensibility needs lifecycle
   visibility, not only protocol calls.

### Locality

Keep behavior near its owning domain. File-name inference belongs in
`nako-library`/`nako-naming`; provider graph and mapping policy in
`nako-metadata`; catalog hydration/search in `nako-catalog`/`nako-search`;
playback decisions in `nako-playback`; FFmpeg execution in `nako-transcode`;
HTTP translation in `nako-server`/`nako-api`; client DTO compatibility in
`nako-client-protocol`. This avoids the mature-server failure mode where API,
DB, provider, playback, and UI assumptions cross-cut each other invisibly.

## Paths Actually Viewed

Jellyfin reference paths:

- `repo-ref/jellyfin`
- `repo-ref/jellyfin/Jellyfin.sln`
- `repo-ref/jellyfin/Directory.Build.props`
- `repo-ref/jellyfin/Directory.Packages.props`
- `repo-ref/jellyfin/Emby.Naming`
- `repo-ref/jellyfin/Emby.Naming/Audio`
- `repo-ref/jellyfin/Emby.Naming/AudioBook`
- `repo-ref/jellyfin/Emby.Naming/Book`
- `repo-ref/jellyfin/Emby.Naming/Common`
- `repo-ref/jellyfin/Emby.Naming/ExternalFiles`
- `repo-ref/jellyfin/Emby.Naming/TV`
- `repo-ref/jellyfin/Emby.Naming/Video`
- `repo-ref/jellyfin/Emby.Server.Implementations`
- `repo-ref/jellyfin/Emby.Server.Implementations/Emby.Server.Implementations.csproj`
- `repo-ref/jellyfin/Emby.Server.Implementations/AppBase`
- `repo-ref/jellyfin/Emby.Server.Implementations/ApplicationHost.cs`
- `repo-ref/jellyfin/Emby.Server.Implementations/Data`
- `repo-ref/jellyfin/Emby.Server.Implementations/Dto`
- `repo-ref/jellyfin/Emby.Server.Implementations/HttpServer`
- `repo-ref/jellyfin/Emby.Server.Implementations/HttpServer/Security`
- `repo-ref/jellyfin/Emby.Server.Implementations/IO`
- `repo-ref/jellyfin/Emby.Server.Implementations/Library`
- `repo-ref/jellyfin/Emby.Server.Implementations/Library/Resolvers`
- `repo-ref/jellyfin/Emby.Server.Implementations/Library/SimilarItems`
- `repo-ref/jellyfin/Emby.Server.Implementations/Library/Validators`
- `repo-ref/jellyfin/Emby.Server.Implementations/Plugins`
- `repo-ref/jellyfin/Emby.Server.Implementations/QuickConnect`
- `repo-ref/jellyfin/Emby.Server.Implementations/ScheduledTasks`
- `repo-ref/jellyfin/Emby.Server.Implementations/Session`
- `repo-ref/jellyfin/Emby.Server.Implementations/Updates`
- `repo-ref/jellyfin/Jellyfin.Api`
- `repo-ref/jellyfin/Jellyfin.Api/Jellyfin.Api.csproj`
- `repo-ref/jellyfin/Jellyfin.Api/Auth`
- `repo-ref/jellyfin/Jellyfin.Api/Controllers`
- `repo-ref/jellyfin/Jellyfin.Api/Helpers`
- `repo-ref/jellyfin/Jellyfin.Api/Middleware`
- `repo-ref/jellyfin/Jellyfin.Api/Models`
- `repo-ref/jellyfin/Jellyfin.Api/Results`
- `repo-ref/jellyfin/Jellyfin.Api/WebSocketListeners`
- `repo-ref/jellyfin/Jellyfin.Data`
- `repo-ref/jellyfin/Jellyfin.Data/Enums`
- `repo-ref/jellyfin/Jellyfin.Data/Events`
- `repo-ref/jellyfin/Jellyfin.Data/Queries`
- `repo-ref/jellyfin/Jellyfin.Server`
- `repo-ref/jellyfin/Jellyfin.Server/Jellyfin.Server.csproj`
- `repo-ref/jellyfin/Jellyfin.Server/CoreAppHost.cs`
- `repo-ref/jellyfin/Jellyfin.Server/Program.cs`
- `repo-ref/jellyfin/Jellyfin.Server/Startup.cs`
- `repo-ref/jellyfin/Jellyfin.Server/Configuration`
- `repo-ref/jellyfin/Jellyfin.Server/Filters`
- `repo-ref/jellyfin/Jellyfin.Server/HealthChecks`
- `repo-ref/jellyfin/Jellyfin.Server/Migrations`
- `repo-ref/jellyfin/Jellyfin.Server/ServerSetupApp`
- `repo-ref/jellyfin/Jellyfin.Server/wwwroot/api-docs`
- `repo-ref/jellyfin/Jellyfin.Server.Implementations`
- `repo-ref/jellyfin/Jellyfin.Server.Implementations/Activity`
- `repo-ref/jellyfin/Jellyfin.Server.Implementations/Devices`
- `repo-ref/jellyfin/Jellyfin.Server.Implementations/Events`
- `repo-ref/jellyfin/Jellyfin.Server.Implementations/FullSystemBackup`
- `repo-ref/jellyfin/Jellyfin.Server.Implementations/Item`
- `repo-ref/jellyfin/Jellyfin.Server.Implementations/MediaSegments`
- `repo-ref/jellyfin/Jellyfin.Server.Implementations/Security`
- `repo-ref/jellyfin/Jellyfin.Server.Implementations/Trickplay`
- `repo-ref/jellyfin/Jellyfin.Server.Implementations/Users`
- `repo-ref/jellyfin/MediaBrowser.Common`
- `repo-ref/jellyfin/MediaBrowser.Controller`
- `repo-ref/jellyfin/MediaBrowser.Controller/MediaBrowser.Controller.csproj`
- `repo-ref/jellyfin/MediaBrowser.Controller/Authentication`
- `repo-ref/jellyfin/MediaBrowser.Controller/Devices`
- `repo-ref/jellyfin/MediaBrowser.Controller/Entities`
- `repo-ref/jellyfin/MediaBrowser.Controller/Events`
- `repo-ref/jellyfin/MediaBrowser.Controller/Library`
- `repo-ref/jellyfin/MediaBrowser.Controller/LiveTv`
- `repo-ref/jellyfin/MediaBrowser.Controller/MediaEncoding`
- `repo-ref/jellyfin/MediaBrowser.Controller/MediaSegments`
- `repo-ref/jellyfin/MediaBrowser.Controller/Persistence`
- `repo-ref/jellyfin/MediaBrowser.Controller/Plugins`
- `repo-ref/jellyfin/MediaBrowser.Controller/Providers`
- `repo-ref/jellyfin/MediaBrowser.Controller/Resolvers`
- `repo-ref/jellyfin/MediaBrowser.Controller/Security`
- `repo-ref/jellyfin/MediaBrowser.Controller/Session`
- `repo-ref/jellyfin/MediaBrowser.Controller/Streaming`
- `repo-ref/jellyfin/MediaBrowser.Controller/Trickplay`
- `repo-ref/jellyfin/MediaBrowser.LocalMetadata`
- `repo-ref/jellyfin/MediaBrowser.MediaEncoding`
- `repo-ref/jellyfin/MediaBrowser.MediaEncoding/Attachments`
- `repo-ref/jellyfin/MediaBrowser.MediaEncoding/BdInfo`
- `repo-ref/jellyfin/MediaBrowser.MediaEncoding/Configuration`
- `repo-ref/jellyfin/MediaBrowser.MediaEncoding/Encoder`
- `repo-ref/jellyfin/MediaBrowser.MediaEncoding/Probing`
- `repo-ref/jellyfin/MediaBrowser.MediaEncoding/Subtitles`
- `repo-ref/jellyfin/MediaBrowser.MediaEncoding/Transcoding`
- `repo-ref/jellyfin/MediaBrowser.Model`
- `repo-ref/jellyfin/MediaBrowser.Model/Configuration`
- `repo-ref/jellyfin/MediaBrowser.Model/Dlna`
- `repo-ref/jellyfin/MediaBrowser.Model/Dto`
- `repo-ref/jellyfin/MediaBrowser.Model/Entities`
- `repo-ref/jellyfin/MediaBrowser.Model/Library`
- `repo-ref/jellyfin/MediaBrowser.Model/LiveTv`
- `repo-ref/jellyfin/MediaBrowser.Model/MediaInfo`
- `repo-ref/jellyfin/MediaBrowser.Model/MediaSegments`
- `repo-ref/jellyfin/MediaBrowser.Model/Plugins`
- `repo-ref/jellyfin/MediaBrowser.Model/Session`
- `repo-ref/jellyfin/MediaBrowser.Model/System`
- `repo-ref/jellyfin/MediaBrowser.Model/Tasks`
- `repo-ref/jellyfin/MediaBrowser.Model/Updates`
- `repo-ref/jellyfin/MediaBrowser.Model/Users`
- `repo-ref/jellyfin/MediaBrowser.Providers`
- `repo-ref/jellyfin/MediaBrowser.Providers/Manager`
- `repo-ref/jellyfin/MediaBrowser.Providers/MediaInfo`
- `repo-ref/jellyfin/MediaBrowser.Providers/Plugins`
- `repo-ref/jellyfin/MediaBrowser.Providers/Subtitles`
- `repo-ref/jellyfin/MediaBrowser.Providers/Trickplay`
- `repo-ref/jellyfin/MediaBrowser.XbmcMetadata`
- `repo-ref/jellyfin/src/Jellyfin.Database`
- `repo-ref/jellyfin/src/Jellyfin.Database/Jellyfin.Database.Implementations`
- `repo-ref/jellyfin/src/Jellyfin.Database/Jellyfin.Database.Providers.Sqlite`
- `repo-ref/jellyfin/src/Jellyfin.LiveTv`
- `repo-ref/jellyfin/src/Jellyfin.MediaEncoding.Hls`
- `repo-ref/jellyfin/src/Jellyfin.MediaEncoding.Keyframes`
- `repo-ref/jellyfin/tests`
- `repo-ref/jellyfin/tests/Jellyfin.Api.Tests`
- `repo-ref/jellyfin/tests/Jellyfin.MediaEncoding.Tests`
- `repo-ref/jellyfin/tests/Jellyfin.Naming.Tests`
- `repo-ref/jellyfin/tests/Jellyfin.Providers.Tests`
- `repo-ref/jellyfin/tests/Jellyfin.Server.Integration.Tests`

Nako context paths used for comparison:

- `CONTEXT.md`
- `docs/ARCHITECTURE.md`
- `docs/ROADMAP.md`
- `docs/GOALS.md`
- `docs/architecture/LIBRARY_PIPELINE.md`
- `docs/architecture/PLAYBACK.md`
- `docs/architecture/CONTROL_PLANE.md`
- `docs/architecture/STATE_ACCESS.md`
- `docs/architecture/OPERATIONS_RELEASE.md`
- `docs/architecture/REALTIME_SYNC.md`
- `.trellis/tasks/06-10-media-server-gap-analysis/prd.md`
- `.trellis/spec/guides/index.md`
- `.trellis/spec/guides/cross-layer-thinking-guide.md`
- `.trellis/spec/guides/code-reuse-thinking-guide.md`
- `crates/nako-core/src`
- `crates/nako-library/src`
- `crates/nako-metadata/src`
- `crates/nako-playback/src`
- `crates/nako-transcode/src`
- `crates/nako-streaming/src`
- `crates/nako-server/src`
- `crates/nako-api/src`
- `crates/nako-client-protocol/src`
- `crates/nako-addon-protocol/src`
- `apps/admin-web/src`
