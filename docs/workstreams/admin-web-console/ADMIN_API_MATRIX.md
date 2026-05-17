# Admin API Matrix

Status: Draft
Last updated: 2026-05-18

This document records AWC-020: which current HTTP routes can support the Taru
admin web console, which routes are public-client surfaces that the console can
reuse read-only, and which Admin API gaps remain.

It is an inventory, not an API compatibility promise. AWC-030 is accepted in
[ADR 0027](../../adr/0027-admin-api-boundary-for-web-console.md): new
admin-only routes should use `/admin/v1/*`, admin DTOs stay in `taru-api`, and
Public Client API contracts remain separate.

## Source Files Reviewed

- `crates/taru-server/src/http.rs`
- `crates/taru-server/src/http/system.rs`
- `crates/taru-server/src/http/library.rs`
- `crates/taru-server/src/http/catalog.rs`
- `crates/taru-server/src/http/metadata.rs`
- `crates/taru-server/src/http/playback.rs`
- `crates/taru-server/src/http/jobs.rs`
- `crates/taru-server/src/http/webhooks.rs`
- `crates/taru-server/src/http/automation.rs`
- `crates/taru-server/src/http/addons.rs`
- `crates/taru-api/src/public_client.rs`
- `crates/taru-api/src/admin.rs`
- `crates/taru-api/src/metadata_diagnostics.rs`
- `crates/taru-api/src/extension.rs`
- `docs/api/HTTP_API.md`

## Current Route Groups

| Area | Current routes | Surface | Console fit |
| --- | --- | --- | --- |
| Health | `GET /health` | Public Client API | Directly supports server status and API version. |
| Admin overview | `GET /admin/v1/overview` | Admin API v1 | First read-only overview summary for server/API version, storage status, metadata provider status, runtime counters, and startup counters. |
| Libraries | `GET /libraries`, `GET /libraries/{library_id}`, `GET /libraries/{library_id}/sources` | Public Client API | Directly supports library list/detail and source list, but uses client-facing DTOs. |
| Library operations | `POST /libraries/{library_id}/scan`, `POST /libraries/{library_id}/nfo/import`, `POST /libraries/{library_id}/nfo/export` | Admin/internal | Supports scan and NFO job actions. |
| Ingestion failures | `GET/POST /libraries/{library_id}/ingestion/failures` | Admin/internal | Supports failure list and ignore action. Missing retry/resolution actions. |
| Catalog browse | `GET /items`, `GET /items/{item_id}`, `GET /items/{item_id}/credits`, `GET /items/{item_id}/images`, people/tags/genres/search routes | Public Client API | Supports read-only catalog review and item detail. Not enough for repair workflows. |
| Source probe | `GET /sources/{source_id}/probe` | Public Client API | Supports source technical-facts panel. |
| Metadata jobs | `POST /items/{item_id}/metadata/refresh`, `POST /metadata/maintenance/jobs`, `POST /metadata/maintenance/plan` | Admin/internal | Supports item refresh, batch maintenance enqueue, and dry-run planning. |
| Metadata diagnostics | `GET /items/{item_id}/metadata/attempts`, `GET /items/{item_id}/metadata/raw`, `GET /metadata/providers`, `POST /metadata/raw/cleanup` | Admin/internal | Supports provider status, item attempts, raw cache view, and cleanup. |
| Playback decisions | `GET /sources/{source_id}/playback/decision` | Public Client API | Supports decision preview for a single source. |
| Streaming playback | `GET/HEAD /sources/{source_id}/stream`, `GET /sources/{source_id}/stream/remux`, `GET /sources/{source_id}/stream/hls/playlist.m3u8`, HLS segment route | Public Client API | Not a first admin-console need except safe diagnostics and request previews. |
| Playback sessions | `GET /admin/v1/playback/sessions`, `GET /playback/sessions/{session_id}`, `POST /playback/sessions/{session_id}/cancel` | Admin API v1 plus Public Client API detail/cancel | Supports redacted admin list/filter by state, kind, Media Source, and pagination. Existing Public Client API supports detail/cancel only when the session ID is already known. |
| Jobs | `GET /admin/v1/jobs`, `GET /jobs/{job_id}` | Admin API v1 plus legacy admin/internal detail | Supports redacted list/filter by status, kind, resource class, Media Library, Media Source, and pagination. Supports detail only when job ID is known. Missing cancel/retry. |
| Storage diagnostics | `GET /storage/backends` | Admin/internal | Supports storage page read-only diagnostics. |
| Webhooks | `POST /webhooks/endpoints`, `GET /webhooks/endpoints`, `GET /webhooks/endpoints/{endpoint_id}`, `GET /events/{event_id}/webhook-attempts`, `POST /events/{event_id}/webhooks/deliver` | Admin/internal | Supports endpoint upsert/list/detail, event attempt detail, and explicit dispatch. Missing event list and disabled endpoint listing semantics need review. |
| Automation | `POST /automation/providers`, `GET /automation/providers`, `GET /automation/providers/{provider_id}`, `POST /automation/jobs`, `GET /automation/jobs/{job_id}/artifacts`, `GET /items/{item_id}/automation/artifacts` | Admin/internal | Supports provider upsert/list/detail, job enqueue, artifact inspection. Missing all-provider list semantics, job list, and artifact approval/rejection lifecycle. |
| Addons | `POST /addons`, `GET /addons`, `GET /addons/{addon_id}` | Admin/internal | Supports registration/list/detail/status filter. Missing health check, token rotation, revoke/delete, and resource-call diagnostics. |

## Page Coverage Matrix

| Console page | Current support | Missing or weak Admin API |
| --- | --- | --- |
| Overview | Good for first read-only summary: `GET /admin/v1/overview` composes health/version, storage backend status, metadata provider status, runtime counters, and startup recovery counters. Existing `GET /health`, `GET /metadata/providers`, and `GET /storage/backends` remain available. `GET /admin/v1/jobs` and `GET /admin/v1/playback/sessions` support drill-down tables. | Still needs outbox events, recent failures, and warning list/filter endpoints if the console needs more drill-down data. |
| Media Libraries | Good for read and actions: library list/detail/sources, scan, NFO import/export, ingestion failure list/ignore. | Needs create/edit/delete library only if Taru supports runtime-configurable libraries. Needs failure retry/resolve semantics if desired. |
| Library Detail | Good for core read-only detail and operations. | Needs latest scan summary, configured backend detail without unsafe local paths, and per-library job history. |
| Catalog | Partial: public browse/search/item/credits/images/source probe. | Needs unknown-item filter, duplicate-source list, provider mapping list, local inference evidence route, hierarchy repair routes, and source variant/edition governance. |
| Item Detail | Partial: item, sources through library source list, credits, images, source probe, metadata attempts/raw, automation artifacts. | Needs direct item-to-sources route, provider mappings, local inference evidence, NFO sidecar status, field provenance/field locks, duplicate relationships, and admin-only source diagnostics. |
| Metadata Providers | Good: provider diagnostics and runtime budgets exist. | Needs configuration edit if provider config becomes runtime editable. Current diagnostics are process-local. |
| Metadata Maintenance | Good: dry-run plan, enqueue job, item refresh, raw cache cleanup. | Needs maintenance schedule read/edit routes if schedules become UI-managed. |
| Jobs/Tasks | Good for first read-only list: `GET /admin/v1/jobs` supports redacted list/filter by status/kind/library/source/resource class and pagination. Existing `GET /jobs/{job_id}` supports known-ID detail. | Needs retry/cancel only after durable runtime semantics support them. |
| Playback & Transcode | Good for first read-only session list: `GET /admin/v1/playback/sessions` supports redacted list/filter by state/kind/source and pagination. Public playback decision by source, known-session detail/cancel, and streaming routes remain available. | Needs transcode hardware capability report, selected hardware policy, resource budget summary, FFmpeg status, staging budget/cleanup summary, and safe request preview. |
| Storage | Good for first read-only page: storage backend diagnostics. | Needs staging manifest list/cleanup diagnostics if storage page includes cache/staging operations. |
| Automation | Partial: provider list/detail/upsert, job enqueue, artifact list by job/item. | Needs all-provider list including disabled if current list remains enabled-only, job list/filter, artifact approval/reject/apply lifecycle, and provider health checks. |
| Webhooks | Partial: endpoint upsert/list/detail, delivery attempts by event, manual delivery by event. | Needs event outbox list/detail route; endpoint list currently reads as enabled-only and may not support disabled endpoint administration. |
| Addons | Partial: register/list/detail/status-filtered list. | Needs health check, token rotation/revocation, enable/disable patch route if status update through full register is too coarse, delete/unregister, hosted-page metadata, and resource-call diagnostics. |
| Network | Mostly missing. | Needs self-hosted access summary, external reachability probe, reverse proxy/TLS status hooks, tunnel/NAT traversal state, and remote playback bandwidth policy. |
| Settings | Mostly missing. | Needs sanitized config summary, auth status, secret-reference diagnostics, FFmpeg path/status, maintenance schedule summary, startup report, backup/database status if exposed. |

## DTO Ownership Notes

Current `taru-api` ownership already separates several useful groups:

- `public_client.rs`: route/DTO contract for future clients and SDKs.
- `admin.rs`: `JobResponse`, ingestion failures, and storage diagnostics.
- `metadata_diagnostics.rs`: provider attempts, raw cache, provider runtime
  diagnostics, metadata maintenance request/plan/cleanup DTOs.
- `extension.rs`: webhook, automation, and addon request/response DTOs.

For the admin console, this split is useful but incomplete. The likely next
Admin API DTO additions should stay out of `taru-client-protocol` unless they
become genuine public client features.

Likely new Admin DTO groups:

- `AdminOverviewResponse`
- `AdminJobListResponse` and redacted `AdminJobListItem`
- `AdminPlaybackSessionListResponse` and redacted
  `AdminPlaybackSessionListItem`
- `OutboxEventListResponse` and `OutboxEventResponse`
- `TranscodeRuntimeDiagnosticsResponse`
- `ServerConfigDiagnosticsResponse`
- `StartupReportResponse`
- `CatalogGovernanceResponse` or narrower DTOs for unknown items, duplicate
  relationships, local inference evidence, provider mappings, and NFO status
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

1. **Overview follow-up slice**: M52 added the first read-only
   `GET /admin/v1/overview` summary. M54 added `GET /admin/v1/jobs`. M55
   added `GET /admin/v1/playback/sessions`. Add list/filter endpoints for
   outbox events and recent failures when the console needs more drill-down
   data.
2. **Playback diagnostics slice**: expose hardware acceleration report,
   selected policy, FFmpeg availability, transcode resource budget, and
   staging budget summary without local output paths.
3. **Catalog governance slice**: expose unknown items, provider mappings,
   local inference evidence, duplicate-source relationships, and NFO sidecar
   status before adding repair mutations.
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
- webhook/automation/addon registration views;
- job list/filter through `GET /admin/v1/jobs`;
- playback session list/filter through `GET /admin/v1/playback/sessions`;
- job/session detail views only when seeded with known IDs or mocked data.

Event lists, hardware capability dashboards, network checks, settings editing,
and catalog repair should remain prototype/mock states until follow-up Admin
API work lands.

After M52, the overview page can use `GET /admin/v1/overview` for its compact
server, storage, metadata-provider, runtime, and startup summary. After M54,
Jobs/Tasks can use `GET /admin/v1/jobs` for redacted list/filter data. Other
After M55, Playback & Transcode can use `GET /admin/v1/playback/sessions` for
redacted session list/filter data. Other drill-down tables and operational
histories remain mock or follow-up Admin API work.
