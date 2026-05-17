# v0 Context For Taru Admin Console

Status: Draft
Last updated: 2026-05-17

Use this document as product context for generating a first Taru web admin
console prototype. It intentionally does not choose a front-end framework,
component library, data-fetching library, or exact visual implementation.

## Project Summary

Taru is a self-hosted media server backend written as a Rust workspace. It is
inspired by the self-hosted media-library category of Jellyfin and Plex, but it
is not trying to clone either product.

Taru's first web surface should be an administration console for the server. It
should help an operator configure media libraries, inspect media catalog
quality, diagnose metadata and playback behavior, manage automation, and
understand the health of a private server.

This is not the flagship playback client. Future playback clients may be
mobile, native, TV, Flutter, or other client applications. The web console may
include light browsing and item inspection, but its main job is administration
and media governance.

## Brand Direction

The name "Taru" comes from Japanese "樽", meaning wooden barrel or sake cask.
The brand metaphor is a private media cellar: classic anime, films, series,
and personal media are preserved carefully like aged bottles in a small cask.

Design personality:

- private and self-hosted;
- refined and calm;
- preservation-oriented;
- transparent and trustworthy;
- technical enough for administrators, but not hostile;
- more like a quiet cellar control room than a streaming-service storefront.

Visual direction:

- professional admin console;
- restrained density;
- clear tables, filters, tabs, status badges, and detail drawers;
- avoid a marketing landing page;
- avoid a poster-wall-only media app;
- use a warm accent inspired by wood or aged copper, but do not make the whole
  UI brown, beige, or monochrome;
- the product name Taru should be visible in the shell;
- use concise labels and actionable empty/error states.

## Domain Language

Use these Taru terms in UI labels and page text:

- Media Library: a configured collection boundary.
- Media Source: one playable file or remote object inside a library.
- Media Item: a user-facing entry such as movie, series, season, or episode.
- Canonical Metadata: the metadata Taru uses for browsing and playback.
- Provider Mapping: relation between a Media Item and TMDB, Douban, Bangumi,
  or another provider subject.
- Local Inference: low-confidence evidence inferred from paths, file names, and
  local media facts.
- NFO: sidecar metadata file imported from or exported to the library.
- Playback Source Selection: server decision for direct play, remux, or HLS
  transcode.
- Addon Sidecar: external service implementing Taru's addon protocol.
- Automation Provider: external provider used for automation or AI-like tasks.

Avoid making the UI provider-centric or file-manager-centric when a Taru term
exists.

## Server Capabilities To Represent

Taru backend capabilities include:

- media libraries backed by local storage and remote storage such as WebDAV;
- library scan jobs;
- catalog browse and search;
- metadata refresh using providers such as TMDB, Douban, and Bangumi;
- NFO import/export and local metadata authority;
- raw provider response cache for diagnostics;
- metadata provider attempts, failures, rate limits, and availability;
- media technical facts from probes;
- playback decision for direct play, remux, or HLS transcode;
- FFmpeg-backed remux and HLS sessions;
- hardware acceleration capability for VAAPI, NVENC, and QuickSync;
- remote direct play and remote staging resource budgets;
- storage backend diagnostics;
- event outbox and webhooks;
- automation providers, jobs, and generated artifacts;
- HTTP sidecar addons with manifests, resources, scopes, and enabled/disabled
  status;
- bearer-token admin access.

## API Boundary

Taru has two API surfaces:

- Public Client API: stable client-facing routes described by the public
  protocol, OpenAPI, and SDK workstreams.
- Admin API: future web-console routes accepted by ADR 0027 under
  `/admin/v1/*`.

For a prototype, do not imply that every page already has a live route. Some
data should be mocked until follow-up Admin API slices exist. Current public
client routes may support read-only library, catalog, source, playback
decision, and session-detail views. Admin-only diagnostics, overview rollups,
job lists, event lists, hardware dashboards, catalog repair queues, settings
editing, and extension lifecycle operations should be treated as mock or
planned `/admin/v1/*` data unless explicitly wired later.

Do not put admin-only DTOs into Public Client API language. Do not describe
`taru-client-protocol` as the source for admin console diagnostics.

## Primary Navigation

Use a left-side app shell with these primary sections:

- Overview
- Media Libraries
- Catalog
- Metadata
- Playback & Transcode
- Storage
- Automation
- Addons
- Network
- Settings

The shell should support deep administrative workflows without feeling like a
marketing website.

## Suggested Routes

The exact routing technology is not fixed. Use these as product routes:

```text
/overview
/libraries
/libraries/new
/libraries/:libraryId
/libraries/:libraryId/sources
/catalog
/catalog/unknown
/catalog/duplicates
/items/:itemId
/items/:itemId/metadata
/items/:itemId/sources
/metadata/providers
/metadata/maintenance
/metadata/raw-cache
/playback/sessions
/playback/sessions/:sessionId
/playback/decision-preview
/transcode
/storage
/automation/webhooks
/automation/events/:eventId
/automation/providers
/automation/jobs
/automation/jobs/:jobId
/addons
/addons/new
/addons/:addonId
/network
/settings
```

## Page Requirements

### Overview

Show a compact operational dashboard:

- server health;
- active jobs;
- recent failures;
- provider availability;
- storage backend status;
- active playback/transcode sessions;
- webhook, automation, and addon warnings.

### Media Libraries

Show libraries as manageable server resources:

- list of libraries;
- create library flow;
- library detail;
- scan action;
- NFO import/export actions;
- storage backend capability summary;
- latest scan and ingestion failure state.

### Catalog

Show catalog governance:

- searchable media item table/grid;
- unknown media queue;
- duplicate source review;
- item detail with sources, images, tags, genres, credits, provider mappings,
  and technical facts;
- repair-oriented actions can be placeholders if not implemented yet.

### Metadata

Show metadata operations and provenance:

- provider status for TMDB, Douban, Bangumi, NFO, and local inference;
- provider attempts and retryable failures;
- metadata maintenance planning and job creation;
- raw cache cleanup;
- per-item metadata diagnostics.

### Playback & Transcode

Show playback runtime behavior:

- active and historical sessions;
- session state: planned, starting, running, cancel requested, finished,
  failed, cancelled;
- direct play, remux, and HLS categories;
- hardware acceleration summary for VAAPI, NVENC, and QuickSync;
- concurrency and staging budget cards;
- playback decision preview.

### Storage

Show storage diagnostics:

- backend per library;
- local or remote kind;
- readable/writable capabilities;
- cache state;
- remote stream/stage budget;
- timeout, unauthorized, rate-limited, or stale-cache indicators.

### Automation

Show external automation:

- webhook endpoints;
- event delivery attempts;
- automation providers;
- automation jobs;
- generated artifacts attached to items.

### Addons

Show Addon Sidecars:

- addon registration list;
- manifest summary;
- protocol version;
- resources;
- granted scopes;
- enabled/disabled status;
- health placeholder;
- hosted addon pages as external links, clearly not trusted admin UI.

### Network

Show self-hosted access:

- local server URL;
- external access status placeholder;
- reverse proxy guidance placeholder;
- HTTPS/TLS status placeholder;
- tunnel/NAT traversal placeholder;
- remote playback bandwidth summary placeholder.

### Settings

Show server configuration:

- admin auth status;
- secret references and missing environment diagnostics;
- FFmpeg path/status;
- transcode policy;
- maintenance schedules;
- logging and diagnostics actions.

## Data And Safety Rules

- Do not show plaintext secrets.
- Do not show tokens.
- Do not show resolved provider API keys.
- Do not show webhook signing secrets.
- Do not show addon bearer tokens.
- Prefer secret reference labels such as environment variable names.
- Redact sensitive request headers.
- Use safe diagnostic messages.
- Raw provider bodies may be shown only in explicit diagnostics views.
- Hosted addon pages are external and not trusted admin UI.

## Desired Interaction Patterns

- Tables with filters for jobs, attempts, sources, sessions, and endpoints.
- Status badges for health, enabled/disabled, retryable, read-only, writable,
  running, failed, and rate-limited states.
- Detail drawers or detail pages for media items, jobs, sessions, addons, and
  webhook attempts.
- Dry-run before destructive or broad metadata operations.
- Clear "Retry", "Run scan", "Refresh metadata", "Cancel session", "Copy safe
  diagnostics", and "Open job" actions.
- Empty states should explain the next useful action.
- Error states should be actionable and safe.

## First Prototype Scope

Generate a prototype with these pages first:

- Overview
- Media Libraries
- Library Detail
- Metadata Providers
- Jobs/Tasks view
- Playback & Transcode
- Settings

It is acceptable to use realistic mock data. The prototype should make the
information architecture and product language clear before real API wiring.

Mock-only or planned `/admin/v1/*` prototype areas should be visually marked as
diagnostic or planned data in internal handoff notes, not as existing stable
Public Client API coverage.
