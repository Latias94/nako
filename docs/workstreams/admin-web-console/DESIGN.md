# Admin Web Console Design Baseline

Status: Proposed
Last updated: 2026-05-17

## Problem

Taru has a backend-oriented Rust workspace with many server capabilities
already documented or implemented, but no coherent web administration product
shape. A generated front-end prototype would drift quickly if it only sees a
generic "media server dashboard" prompt.

The console needs to express Taru's actual domain:

- **Media Library** configuration and scan operation;
- **Media Source** and **Media Item** governance;
- **Canonical Metadata** provenance and provider diagnostics;
- NFO import/export and local metadata authority;
- playback decision, remux, HLS, and hardware acceleration diagnostics;
- storage backend health and remote-source constraints;
- webhook, automation, and **Addon Sidecar** operations;
- self-hosted access, secrets, and operational safety.

## Target State

Create a durable planning lane that lets Taru hand a clear product brief to a
front-end generator such as v0.dev, while keeping implementation choices open.

The target output is:

- a stable admin-console scope;
- a page and route inventory;
- a brand direction;
- an Admin API gap list;
- a v0.dev context document that can be copied into a design-generation flow;
- a task ledger for turning the generated UI into a real Taru web surface.

## Product Positioning

Taru is a private media cellar, not a huge warehouse. The Japanese meaning of
"taru" as a wooden barrel or sake cask should guide the brand:

- personal rather than corporate;
- refined rather than maximalist;
- preservation-focused rather than feed-driven;
- transparent and inspectable rather than magical;
- self-hosted and privacy-first.

The admin console should feel like a quiet control room for a careful
collector. It should not mimic a streaming-service storefront, a cloud SaaS
landing page, or a poster-wall-only media app.

## Console Role

The first web console is administration-first:

- configure and monitor the server;
- govern catalog and metadata quality;
- inspect operational failures;
- prepare and review automation results;
- manage extension and integration boundaries.

It is not the flagship playback client. Light item browsing and source
inspection are in scope because they support administration, but polished
watching, TV navigation, offline playback, and native player behavior remain
future client work.

## Page Families

### Overview

Purpose: give the administrator a fast status read.

Expected content:

- server health and API version;
- active scans, metadata jobs, NFO jobs, playback sessions, and automation
  jobs;
- recent failures grouped by actionable category;
- metadata provider availability;
- storage backend status;
- hardware acceleration availability;
- webhook/addon/automation warning summary.

### Media Libraries

Purpose: configure and operate **Media Libraries**.

Expected content:

- library list;
- library detail;
- create/edit library flow;
- scan action;
- NFO import/export actions;
- storage backend and writable capability summary;
- local metadata policy summary;
- recent ingestion failures.

### Catalog Governance

Purpose: inspect and repair the catalog.

Expected content:

- media item browse/search for administrative review;
- unknown media queue;
- duplicate source and source variant review;
- provisional hierarchy repair;
- media item detail with sources, technical facts, images, tags, genres,
  credits, provider mappings, and local inference evidence.

### Metadata

Purpose: make metadata provenance and failures explainable.

Expected content:

- provider availability and runtime budgets;
- TMDB, Douban, Bangumi, NFO, and local inference status;
- provider attempt history per item;
- raw provider cache inspection and cleanup;
- metadata maintenance dry-run planning;
- metadata refresh job creation;
- field provenance summary and locked/local-authority fields when available.

### Playback And Transcode

Purpose: explain playback behavior and resource usage.

Expected content:

- active and historical playback sessions;
- direct play, remux, and HLS session states;
- playback decision preview for a source and client capability set;
- FFmpeg availability;
- hardware acceleration capability summary for VAAPI, NVENC, and QuickSync;
- CPU/GPU/remote stream/remote stage concurrency budgets;
- staging disk budget and cleanup status;
- safe failure diagnostics without local output paths or secrets.

### Storage

Purpose: inspect configured storage boundaries.

Expected content:

- per-library storage backend registry;
- local and remote backend kinds;
- read/write capability summary;
- remote stream/stage permit availability;
- cache state and health counters;
- timeout, unauthorized, and rate-limit failure summaries.

### Automation

Purpose: operate external automation safely.

Expected content:

- webhook endpoints;
- event delivery attempts;
- automation providers;
- automation jobs;
- generated artifacts attached to media items;
- retry and dispatch actions.

### Addons

Purpose: manage **Addon Sidecars** without treating them as trusted in-process
plugins.

Expected content:

- registered addons;
- manifest validation result;
- protocol version;
- granted scopes;
- enabled/disabled status;
- resource list;
- health status when available;
- token rotation and revoke actions when implemented;
- hosted page links clearly marked as external addon pages.

### Network And Access

Purpose: help self-hosted users understand access boundaries.

Expected content:

- local server URL;
- external access status when implemented;
- reverse proxy guidance hooks;
- HTTPS/TLS state when detectable;
- future tunnel or NAT traversal setup;
- upload bandwidth and remote playback policy summary.

### Settings

Purpose: manage server-level operational settings.

Expected content:

- inbound admin token/auth status;
- secret references and missing environment-variable diagnostics;
- FFmpeg path and transcode policy;
- logging and diagnostics export hooks;
- maintenance schedules;
- database/backup/migration status when available.

## Route Inventory

Initial route families:

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

Routes are product intent, not a front-end framework contract.

## API Direction

The existing Public Client API is not enough for this console because it
intentionally excludes admin/internal diagnostics. This workstream should lead
to a distinct Admin API design.

Admin API should cover:

- job list/filter/cancel where supported;
- library scan and NFO operations;
- metadata provider diagnostics, attempts, raw cache, maintenance plans;
- storage backend diagnostics;
- playback/transcode session inspection;
- webhook endpoint and delivery inspection;
- automation provider/job/artifact inspection;
- addon registration and inspection;
- server settings and health summaries.

Admin API must preserve these safety rules:

- never return resolved secrets;
- never expose raw local filesystem paths unless a deliberate admin-only
  decision and redaction policy exists;
- separate secret references from secret values;
- redact tokens and authorization headers;
- keep hosted addon pages outside the trusted admin UI boundary;
- make error responses actionable and stable enough for UI branching.

## Non-Goals

- No front-end framework selection in this planning baseline.
- No generated UI code in this workstream's first slice.
- No full playback-client design.
- No mobile, TV, or Flutter implementation plan.
- No addon marketplace or automatic addon installation flow.
- No new backend behavior under this workstream without a follow-up task.
- No copying Jellyfin, Plex, or other reference project UI/source/assets.

## Open Questions

- Should the Admin API be documented as `/admin/*`, `/api/admin/*`, or a
  versioned contract such as `/admin/v0/*`?
- Should the first web implementation live under `apps/admin-web`,
  `web/admin`, or another workspace path?
- Should generated UI start with static mocked data or connect immediately to
  a generated admin SDK?
- Which settings are editable in v1 versus read-only diagnostics?
- How much catalog repair belongs in the first web slice versus a later
  media-governance milestone?
