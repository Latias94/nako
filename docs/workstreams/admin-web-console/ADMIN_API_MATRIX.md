# Admin API Matrix

Status: Draft
Last updated: 2026-05-21

This document records AWC-020: which current HTTP routes can support the Nako
admin web console, which routes are public-client surfaces that the console can
reuse read-only, and which Admin API gaps remain.

It is an inventory, not an API compatibility promise. AWC-030 is accepted in
[ADR 0027](../../adr/0027-admin-api-boundary-for-web-console.md): new
admin-only routes should use `/admin/v1/*`, admin DTOs stay in `nako-api`, and
Public Client API contracts remain separate.

## Source Files Reviewed

- `crates/nako-server/src/http.rs`
- `crates/nako-server/src/http/system.rs`
- `crates/nako-server/src/http/library.rs`
- `crates/nako-server/src/http/catalog.rs`
- `crates/nako-server/src/http/metadata.rs`
- `crates/nako-server/src/http/playback.rs`
- `crates/nako-server/src/http/jobs.rs`
- `crates/nako-server/src/http/webhooks.rs`
- `crates/nako-server/src/http/automation.rs`
- `crates/nako-server/src/http/addons.rs`
- `crates/nako-api/src/public_client.rs`
- `crates/nako-api/src/admin.rs`
- `crates/nako-api/src/metadata_diagnostics.rs`
- `crates/nako-api/src/extension.rs`
- `docs/api/HTTP_API.md`

## Current Route Groups

| Area | Current routes | Surface | Console fit |
| --- | --- | --- | --- |
| Health | `GET /health` | Public Client API | Directly supports server status and API version. |
| Admin overview | `GET /admin/v1/overview` | Admin API v1 | First read-only overview summary for server/API version, storage status, metadata provider status, runtime counters, and startup counters. |
| Libraries | `GET /libraries`, `GET /libraries/{library_id}`, `GET /libraries/{library_id}/sources` | Public Client API | Directly supports library list/detail and source list, but uses client-facing DTOs. |
| Library operations | `POST /libraries/{library_id}/scan`, `POST /libraries/{library_id}/nfo/import`, `POST /libraries/{library_id}/nfo/export` | Admin/internal | Supports scan and NFO job actions. |
| Ingestion failures | `GET/POST /libraries/{library_id}/ingestion/failures` | Admin/internal | Supports failure list and ignore action. Missing retry/resolution actions. |
| Catalog browse/governance | `GET /items`, `GET /items/{item_id}`, `GET /items/{item_id}/credits`, `GET /items/{item_id}/images`, people/tags/genres/search routes, `GET /admin/v1/catalog/governance/items` | Public Client API plus Admin API v1 | Supports read-only catalog review, item detail, and a redacted unknown/low-confidence governance queue. Not enough for repair workflows. |
| Source probe | `GET /sources/{source_id}/probe` | Public Client API | Supports source technical-facts panel. |
| Metadata jobs | `POST /items/{item_id}/metadata/refresh`, `POST /metadata/maintenance/jobs`, `POST /metadata/maintenance/plan` | Admin/internal | Supports item refresh, batch maintenance enqueue, and dry-run planning. |
| Metadata diagnostics | `GET /items/{item_id}/metadata/attempts`, `GET /items/{item_id}/metadata/raw`, `GET /metadata/providers`, `POST /metadata/raw/cleanup` | Admin/internal | Supports provider status, item attempts, raw cache view, and cleanup. |
| Playback decisions | `GET /sources/{source_id}/playback/decision` | Public Client API | Supports decision preview for a single source. |
| Streaming playback | `GET/HEAD /sources/{source_id}/stream`, `GET /sources/{source_id}/stream/remux`, `GET /sources/{source_id}/stream/hls/playlist.m3u8`, HLS segment route | Public Client API | Not a first admin-console need except safe diagnostics and request previews. |
| Playback sessions | `GET /admin/v1/playback/sessions`, `GET /admin/v1/playback/runtime`, `GET /playback/sessions/{session_id}`, `POST /playback/sessions/{session_id}/cancel` | Admin API v1 plus Public Client API detail/cancel | Supports redacted admin list/filter by state, kind, Media Source, and pagination. Supports safe runtime diagnostics for hardware acceleration, FFmpeg capability evidence, resource budgets, and staging cleanup configuration. Existing Public Client API supports detail/cancel only when the session ID is already known. |
| Jobs | `GET /admin/v1/jobs`, `GET /jobs/{job_id}` | Admin API v1 plus legacy admin/internal detail | Supports redacted list/filter by status, kind, resource class, Media Library, Media Source, and pagination. Supports detail only when job ID is known. Missing cancel/retry. |
| Storage diagnostics | `GET /storage/backends`, `GET /admin/v1/storage/staging` | Admin/internal plus Admin API v1 | Supports storage backend diagnostics and redacted staging/cache diagnostics without exposing local staging paths, full source URIs, cache URIs, or raw cache errors. |
| Webhooks | `POST /webhooks/endpoints`, `GET /webhooks/endpoints`, `GET /webhooks/endpoints/{endpoint_id}`, `GET /events/{event_id}/webhook-attempts`, `POST /events/{event_id}/webhooks/deliver`, `GET /admin/v1/events` | Admin/internal plus Admin API v1 | Supports endpoint upsert/list/detail, redacted event outbox list/filter, event attempt detail, and explicit dispatch. Disabled endpoint listing semantics still need review. |
| Automation | `POST /automation/providers`, `GET /automation/providers`, `GET /automation/providers/{provider_id}`, `POST /automation/jobs`, `GET /automation/jobs/{job_id}/artifacts`, `GET /items/{item_id}/automation/artifacts` | Admin/internal | Supports provider upsert/list/detail, job enqueue, artifact inspection. Missing all-provider list semantics, job list, and artifact approval/rejection lifecycle. |
| Addons | `POST /admin/v1/addons`, `GET /admin/v1/addons`, `GET /admin/v1/addons/{addon_id}`, `PATCH /admin/v1/addons/{addon_id}/status`, `POST /admin/v1/addons/{addon_id}/unregister`, `POST /admin/v1/addons/{addon_id}/health-check`, `GET /admin/v1/addons/{addon_id}/surfaces`, `POST /admin/v1/addons/{addon_id}/diagnostics/resource-call`, token/grant routes under `/admin/v1/addons/{addon_id}` | Admin API v1 | Supports registration/list/detail/status filter with Admin DTO shielding, explicit enable/disable lifecycle mutation, terminal unregister with audit preservation, redaction-safe sidecar health checks, hosted surface read models, bounded resource-call diagnostics, token issue/list/rotate/revoke, and grant replace/list. |

## Page Coverage Matrix

| Console page | Current support | Missing or weak Admin API |
| --- | --- | --- |
| Overview | Good for first read-only summary: `GET /admin/v1/overview` composes health/version, storage backend status, metadata provider status, runtime counters, and startup recovery counters. Existing `GET /health`, `GET /metadata/providers`, and `GET /storage/backends` remain available. `GET /admin/v1/jobs`, `GET /admin/v1/playback/sessions`, and `GET /admin/v1/events` support drill-down tables. | Still needs recent failures and warning list/filter endpoints if the console needs more drill-down data. |
| Media Libraries | Good for read and actions: library list/detail/sources, scan, NFO import/export, ingestion failure list/ignore. | Needs create/edit/delete library only if Nako supports runtime-configurable libraries. Needs failure retry/resolve semantics if desired. |
| Library Detail | Good for core read-only detail and operations. | Needs latest scan summary, configured backend detail without unsafe local paths, and per-library job history. |
| Catalog | Partial but improved: public browse/search/item/credits/images/source probe plus `GET /admin/v1/catalog/governance/items` for a redacted unknown/low-confidence queue with source counts, Local Inference summary, Provider Mapping counts, and duplicate relationship counts. | Needs duplicate-source list, provider mapping list/detail, local inference evidence detail route, NFO sidecar status, hierarchy repair routes, and source variant/edition governance. |
| Item Detail | Partial: item, sources through library source list, credits, images, source probe, metadata attempts/raw, automation artifacts. | Needs direct item-to-sources route, provider mappings, local inference evidence, NFO sidecar status, field provenance/field locks, duplicate relationships, and admin-only source diagnostics. |
| Metadata Providers | Good: provider diagnostics and runtime budgets exist. | Needs configuration edit if provider config becomes runtime editable. Current diagnostics are process-local. |
| Metadata Maintenance | Good: dry-run plan, enqueue job, item refresh, raw cache cleanup. | Needs maintenance schedule read/edit routes if schedules become UI-managed. |
| Jobs/Tasks | Good for first read-only list: `GET /admin/v1/jobs` supports redacted list/filter by status/kind/library/source/resource class and pagination. Existing `GET /jobs/{job_id}` supports known-ID detail. | Needs retry/cancel only after durable runtime semantics support them. |
| Playback & Transcode | Good for first read-only session and runtime diagnostics: `GET /admin/v1/playback/sessions` supports redacted list/filter by state/kind/source and pagination, and `GET /admin/v1/playback/runtime` supports hardware policy/selection, FFmpeg capability evidence, transcode budgets, remote playback budgets, and staging cleanup configuration. Public playback decision by source, known-session detail/cancel, and streaming routes remain available. | Needs safe request preview, richer session detail, and deeper Playback Source Selection diagnostics after subtitles/audio tracks/HDR/client profiles/Source Variants are modeled. |
| Storage | Good for first read-only page: storage backend diagnostics plus `GET /admin/v1/storage/staging` for redacted staging manifest rows, staging budget/startup cleanup counters, and VFS cache summary counters. | Needs staging cleanup mutation only if the console supports operator-triggered cleanup. Full VFS cache object/failure listing remains intentionally deferred. |
| Automation | Partial: provider list/detail/upsert, job enqueue, artifact list by job/item. | Needs all-provider list including disabled if current list remains enabled-only, job list/filter, artifact approval/reject/apply lifecycle, and provider health checks. |
| Webhooks | Good for first read-only event history: endpoint upsert/list/detail, redacted event outbox list/filter through `GET /admin/v1/events`, delivery attempts by event, and manual delivery by event. | Needs event detail only if list rows are insufficient. Endpoint list currently reads as enabled-only and may not support disabled endpoint administration. |
| Addons | Improved: `/admin/v1/addons` owns register/list/detail/status-filtered list, explicit enable/disable mutation, terminal unregister, health checks, hosted surface read models, bounded resource-call diagnostics, token issue/list/rotate/revoke, and accepted permission grants without exposing raw persistence records. | Addon Manager process/package lifecycle remains out of scope. Future UI may need install guide and marketplace follow-ons. |
| Network | Mostly missing. | Needs self-hosted access summary, external reachability probe, reverse proxy/TLS status hooks, tunnel/NAT traversal state, and remote playback bandwidth policy. |
| Settings | Partial: `GET /admin/v1/system/config` supports sanitized auth, library, runtime, metadata, transcode, staging, and playback config diagnostics. | Needs editable settings only if Nako supports runtime config mutation. FFmpeg binary status is currently covered by playback runtime diagnostics rather than raw path exposure. |

## DTO Ownership Notes

Current `nako-api` ownership already separates several useful groups:

- `public_client.rs`: route/DTO contract for future clients and SDKs.
- `admin.rs`: `JobResponse`, ingestion failures, and storage diagnostics.
- `metadata_diagnostics.rs`: provider attempts, raw cache, provider runtime
  diagnostics, metadata maintenance request/plan/cleanup DTOs.
- `extension.rs`: webhook, automation, and addon request/response DTOs.

For the admin console, this split is useful but incomplete. The likely next
Admin API DTO additions should stay out of `nako-client-protocol` unless they
become genuine public client features.

Likely new Admin DTO groups:

- `AdminOverviewResponse`
- `AdminJobListResponse` and redacted `AdminJobListItem`
- `AdminPlaybackSessionListResponse` and redacted
  `AdminPlaybackSessionListItem`
- `AdminPlaybackRuntimeDiagnosticsResponse`
- `AdminOutboxEventListResponse`
- `AdminServerConfigDiagnosticsResponse`
- `AdminStorageStagingDiagnosticsResponse`
- `StartupReportResponse`
- `AdminCatalogGovernanceItemListResponse` and narrower follow-up DTOs for
  duplicate relationships, local inference evidence detail, provider mappings,
  and NFO status
- `AddonHealthResponse` and `AddonTokenRotationResponse`

## Existing Safety Behavior

Known current safety boundaries useful to preserve:

- Public client OpenAPI tests reject admin/internal surfaces.
- Storage diagnostics avoid remote credentials and secret values.
- Metadata diagnostics expose provider/runtime state but not resolved provider
  secrets or proxy URLs.
- Job inputs and summaries are expected to avoid plaintext secrets.
- Webhook endpoint responses use `secret_env`, not resolved secret values.
- Automation provider config uses `secret_env`, not resolved secret values.
- Addon registration responses include manifest/granted scopes/status, not
  admin tokens.
- Transcode session responses hide local staged output paths.
- Event payload tests check that secrets and local paths are not written into
  outbox payloads.

## Recommended Next Admin API Slices

These are the smallest useful vertical slices after the matrix:

1. **Operations read-model slice**: M57-M59 added `GET /admin/v1/events`,
   `GET /admin/v1/storage/staging`, and `GET /admin/v1/system/config` for
   redacted event outbox history, staging/cache diagnostics, and sanitized
   server config diagnostics. Follow with recent failure/warning list routes
   only when the console needs more overview drill-down data.
2. **Playback diagnostics follow-up**: M56 added
   `GET /admin/v1/playback/runtime` for hardware acceleration report, selected
   policy, FFmpeg capability evidence, transcode resource budget, remote
   playback budgets, and staging cleanup summary without local output paths.
   Follow with safe request preview or richer session detail only when the
   console needs them.
3. **Catalog governance slice**: M60 added
   `GET /admin/v1/catalog/governance/items` for unknown and low-confidence
   Media Items. Follow with provider mapping list/detail, duplicate-source
   review, local inference evidence detail, and NFO sidecar status before
   adding repair mutations.
4. **Extension operations slice**: make Webhook, Automation, and Addon list
   behavior complete for disabled resources and add health/token lifecycle
   operations only after the trust model is documented.

## v0 Context Implication

`V0_CONTEXT.md` can continue to describe the desired page families with mock
data. For the first prototype, API-backed claims should be limited to:

- health;
- library list/detail/sources and scan/NFO actions;
- metadata provider diagnostics and maintenance planning;
- storage backend diagnostics;
- catalog governance queue through `GET /admin/v1/catalog/governance/items`;
- webhook/automation/addon registration views;
- job list/filter through `GET /admin/v1/jobs`;
- event outbox list/filter through `GET /admin/v1/events`;
- playback session list/filter through `GET /admin/v1/playback/sessions`;
- playback runtime diagnostics through `GET /admin/v1/playback/runtime`;
- staging/cache diagnostics through `GET /admin/v1/storage/staging`;
- sanitized config diagnostics through `GET /admin/v1/system/config`;
- job/session detail views only when seeded with known IDs or mocked data.

Network checks, settings editing, and catalog repair should remain
prototype/mock states until follow-up Admin API work lands.

After M52, the overview page can use `GET /admin/v1/overview` for its compact
server, storage, metadata-provider, runtime, and startup summary. After M54,
Jobs/Tasks can use `GET /admin/v1/jobs` for redacted list/filter data. Other
After M55, Playback & Transcode can use `GET /admin/v1/playback/sessions` for
redacted session list/filter data. After M56, Playback & Transcode can use
`GET /admin/v1/playback/runtime` for safe hardware, FFmpeg, budget, and staging
diagnostics. After M57-M59, Automation/Webhooks can use `GET /admin/v1/events`
for redacted event outbox history, Storage can use
`GET /admin/v1/storage/staging` for redacted staging/cache diagnostics, and
Settings can use `GET /admin/v1/system/config` for sanitized configuration
diagnostics. After M60, Catalog can use
`GET /admin/v1/catalog/governance/items` for unknown and low-confidence queue
data. Other drill-down tables and operational histories remain mock or
follow-up Admin API work.
