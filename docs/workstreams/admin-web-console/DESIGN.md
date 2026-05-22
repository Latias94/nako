# Admin Web Console Design Baseline

Status: Completed
Last updated: 2026-05-19

## Problem

Nako has a backend-oriented Rust workspace with many server capabilities
already documented or implemented, but no coherent web administration product
shape. A generated front-end prototype would drift quickly if it only sees a
generic "media server dashboard" prompt.

The console needs to express Nako's actual domain:

- **Media Library** configuration and scan operation;
- **Media Source** and **Media Item** governance;
- **Canonical Metadata** provenance and provider diagnostics;
- NFO import/export and local metadata authority;
- playback decision, remux, HLS, and hardware acceleration diagnostics;
- storage backend health and remote-source constraints;
- webhook, automation, and **Addon Sidecar** operations;
- self-hosted access, secrets, and operational safety.

## Target State

Create a durable planning lane that lets Nako hand a clear product brief to a
front-end generator such as v0.dev, then land the first real admin web
scaffold and live/mock data boundary once the product direction is accepted.

The target output is:

- a stable admin-console scope;
- a page and route inventory;
- a brand direction;
- an Admin API gap list;
- a v0.dev context document that can be copied into a design-generation flow;
- a task ledger for turning the generated UI into a real Nako web surface;
- the first `apps/admin-web` scaffold with safe Admin API reads.

## Product Positioning

Nako is a private media cellar, not a huge warehouse. The Japanese meaning of
"nako" as a wooden barrel or sake cask should guide the brand:

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
/admin/addons
/admin/addons/new
/admin/addons/:addonId
/network
/settings
```

Routes are product intent, not a front-end framework contract.

## Admin API Boundary Decision

The existing Public Client API is not enough for this console because it
intentionally excludes admin/internal diagnostics. This workstream should lead
to a distinct Admin API design.

AWC-030 is accepted through [ADR 0027](../../adr/0027-admin-api-boundary-for-web-console.md):

- new admin-only routes should use `/admin/v1/*`;
- admin DTOs stay in the AGPL `nako-api` adapter boundary, not
  `nako-client-protocol`;
- Public Client API OpenAPI/SDK generation must continue to reject admin
  surfaces;
- the admin console may reuse public routes only for genuinely client-facing
  reads;
- admin routes can be richer than public routes but must still redact secrets,
  tokens, unsafe local paths, raw headers, and implementation-only details.

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

The recommended implementation sequence is:

1. Keep the current route matrix as the source inventory.
2. Add missing admin-only surfaces under `/admin/v1/*`.
3. Wrap or migrate existing root-level admin/internal routes when touched by a
   console slice.
4. Generate any future Admin API contract separately from the Public Client
   OpenAPI/SDK artifacts.
5. Keep redaction and public-route inventory checks as required gates for API
   implementation slices.

M52 / AWC-035 implemented the first slice in that sequence:
`GET /admin/v1/overview`. It is a read-only summary route backed by existing
safe diagnostics. It does not add frontend UI, write mutations, public client
routes, or `nako-client-protocol` changes.

AWC-060 and AWC-070 implemented the first real web app follow-on in
`apps/admin-web`. The app uses Vite, React, and TypeScript, keeps Admin API
reads behind `src/adminApi`, and composes existing `/admin/v1/*` read models
with section-level live/mock fallback. It still needs a generated Admin API
TypeScript contract before deeper route filters, detail pages, or mutations
are added.

## Non-Goals

- No front-end framework selection in this planning baseline.
- No generated UI code in this workstream's first slice.
- No full playback-client design.
- No mobile, TV, or Flutter implementation plan.
- No addon marketplace or automatic addon installation flow.
- No new backend behavior under this workstream without a follow-up task.
- No copying Jellyfin, Plex, or other reference project UI/source/assets.

## Closeout And Splits

- The first web implementation lives under `apps/admin-web`.
- The first implementation uses a deliberate live/mock split: existing
  `/admin/v1/*` reads are live-capable, while missing surfaces stay safe
  deterministic fixtures.
- The generated Admin API TypeScript contract is split to
  `docs/workstreams/admin-api-typescript-contract/`.
- Editable settings remain out of scope until Nako has accepted runtime
  configuration mutation semantics.
- Catalog repair remains out of scope; the current console reads governance
  queues and should add repair workflows only after Admin API mutations are
  explicitly designed.
