# Taru HTTP API

## Scope

This document describes the current server API contract for the local
video-library playback MVP plus the first extension/automation surface. It is
not a complete OpenAPI specification yet, but it fixes response shapes,
pagination rules, job envelopes, playback session envelopes, webhook
inspection envelopes, and error envelopes for current server routes.

## Base Rules

- JSON is the default request and response format.
- IDs are UUID strings.
- Timestamps are stored and returned as UTC strings.
- List endpoints use offset pagination.
- Error responses use a stable JSON envelope.
- Job responses include both durable input and generated summary when present.

## Pagination

List routes accept:

```text
limit:  optional u32, default 50, max 500
offset: optional u64, default 0
```

Invalid pagination returns `400 invalid_input`.

Paginated responses include:

```json
{
  "page": {
    "limit": 50,
    "offset": 0,
    "returned": 12
  }
}
```

`returned` is the number of items in the current response. It is not a total
count. Total counts can be added later when a route needs them and the database
cost is acceptable.

## Error Envelope

Errors use:

```json
{
  "code": "not_found",
  "message": "not found: job 018f..."
}
```

Current codes:

```text
invalid_input
not_found
conflict
unsupported
provider_error
storage_error
ffmpeg_error
staging_budget_exhausted
staging_validation_mismatch
storage_timeout
storage_unauthorized
storage_rate_limited
database_error
```

Database, storage, and provider errors return safe public messages. Detailed
diagnostics belong in structured logs.

## Job Envelope

Jobs use:

```json
{
  "id": "018f0000-0000-7000-8000-000000000001",
  "kind": "library_scan",
  "status": "queued",
  "resource_class": "disk.scan",
  "library_id": "018f0000-0000-7000-8000-000000000002",
  "source_id": null,
  "input": {
    "library_id": "018f0000-0000-7000-8000-000000000002",
    "force": false
  },
  "summary": null,
  "error": null,
  "queued_at": "2026-05-14T00:00:00.000Z",
  "started_at": null,
  "completed_at": null
}
```

`input` is the persisted request payload. It must avoid plaintext secrets.
Future integrations should store secret references instead.

`summary` is written after success. Failed jobs use `error`.

Metadata refresh jobs use the same envelope with kind `metadata_refresh`.
Their input includes the item ID, selected first provider, force flag, and
language. It must not include the resolved provider token.

NFO jobs use kinds `nfo_import` and `nfo_export`. Their input includes the
library ID, local metadata policy, and force flag.

## Current Routes

```text
GET  /health
GET  /libraries?limit=50&offset=0
POST /libraries/{library_id}/scan
POST /libraries/{library_id}/nfo/import
POST /libraries/{library_id}/nfo/export
GET  /libraries/{library_id}/sources?limit=50&offset=0
GET  /items?limit=50&offset=0
GET  /items/{item_id}
GET  /items/{item_id}/credits
GET  /items/{item_id}/images
GET  /people?limit=50&offset=0
GET  /people/{person_id}
GET  /people/{person_id}/items?limit=50&offset=0
GET  /tags?limit=50&offset=0
GET  /tags/{tag_id}/items?limit=50&offset=0
GET  /genres?limit=50&offset=0
GET  /genres/{genre_id}/items?limit=50&offset=0
GET  /search?q=matrix&facet=genre:sci-fi&limit=50&offset=0
POST /items/{item_id}/metadata/refresh
GET  /items/{item_id}/metadata/attempts?limit=50&offset=0
GET  /items/{item_id}/metadata/raw?limit=50&offset=0
GET  /metadata/providers
GET  /sources/{source_id}/probe
GET  /sources/{source_id}/playback/decision
GET  /sources/{source_id}/stream
HEAD /sources/{source_id}/stream
GET  /sources/{source_id}/stream/remux
GET  /sources/{source_id}/stream/hls/playlist.m3u8
GET  /playback/sessions/{session_id}
GET  /playback/sessions/{session_id}/hls/segments/{segment_name}
POST /webhooks/endpoints
GET  /webhooks/endpoints
GET  /webhooks/endpoints/{endpoint_id}
GET  /events/{event_id}/webhook-attempts
POST /events/{event_id}/webhooks/deliver
POST /addons
GET  /addons
GET  /addons/{addon_id}
POST /automation/providers
GET  /automation/providers
GET  /automation/providers/{provider_id}
POST /automation/jobs
GET  /automation/jobs/{job_id}/artifacts
GET  /items/{item_id}/automation/artifacts?limit=50&offset=0
GET  /jobs/{job_id}
```

`POST /libraries/{library_id}/scan` returns `202 Accepted` with a queued job.
The job runs in the background.

`POST /items/{item_id}/metadata/refresh` returns `202 Accepted` with a queued
metadata refresh job. The current implementation uses the library metadata
profile provider order and records provider attempts for the item diagnostics
API.

Metadata diagnostics routes expose provider refresh visibility without exposing
resolved secrets. `GET /items/{item_id}/metadata/attempts` returns persisted
provider attempts with status, failure class, message, matched key, and a
computed `retryable` flag. `GET /items/{item_id}/metadata/raw` returns cached raw
provider responses for the item. `GET /metadata/providers` returns configured
provider availability, sanitized runtime budgets, and whether a proxy is
configured; it never returns token, API key, custom header, or proxy URL values.

`POST /libraries/{library_id}/nfo/import` and
`POST /libraries/{library_id}/nfo/export` return `202 Accepted` with queued NFO
jobs. Import reads same-stem `.nfo` files according to the library local
metadata policy. Export writes sidecars only when the policy is
`write_sidecar`.

Browse routes expose the normalized catalog graph. `GET /items/{item_id}`
returns the media item plus sources, graph relation IDs, and item image assets.
`/credits` returns item credits with the referenced people. People, tags, and
genres list routes are paginated, and their `/items` routes return linked media
items.

`GET /search` reads the SQLite search projection behind `taru-search`. `facet`
is optional and comma-separated for the current lightweight route shape. Search
results return projected media items and relevance scores; richer filters and
ranking can be added behind the same search boundary.

`GET /sources/{source_id}/playback/decision` returns the source, optional probe
data, and a playback decision. Optional query parameters can narrow client
capabilities:

```text
direct_play=true
container=mp4,webm
video_codec=h264,hevc
audio_codec=aac,opus
```

`GET /sources/{source_id}/stream` serves direct play bytes for local sources
and configured WebDAV preview sources. It supports HTTP `Range` requests and
returns `206 Partial Content` with `Accept-Ranges`, `Content-Range`, and
`Content-Length` when a satisfiable range is requested. For WebDAV sources,
Taru streams the selected range through `taru-vfs` into the HTTP response body
instead of buffering the selected bytes in memory.

`HEAD /sources/{source_id}/stream` returns the same direct play headers without
a body. Clients can use it to preflight source length, MIME type, range support,
and range validity before starting playback.

Invalid, unsupported multi-range, or unsatisfiable ranges return
`416 Range Not Satisfiable` with `Content-Range: bytes */{total_len}` and
`Content-Length: 0`.

`GET /sources/{source_id}/stream/remux` runs or reuses a staged copy-remux for
sources whose playback decision is `remux`, then streams the staged output. It
accepts the same client capability query parameters as the playback decision
route plus:

```text
output_container=mp4|mkv
```

The default output container is `mp4`. Completed staged outputs are reused.
Equivalent in-flight remux requests return `409 conflict`. Remux playback
creates a persisted playback session record so state can be inspected after the
request completes or after a server restart. WebDAV sources are staged under
`remux_staging_root/inputs` before FFmpeg is invoked; source locators and
WebDAV credentials are not passed to FFmpeg.

`GET /playback/sessions/{session_id}` returns the persisted session state for
remux and HLS transcode sessions. The response includes the source ID, session
kind, request key, staged output path, state, failure category/message, and
lifecycle timestamps. Missing sessions return `404 not_found`.

`GET /sources/{source_id}/stream/hls/playlist.m3u8` starts or reuses a minimal
single-variant HLS transcode session and returns a rewritten media playlist.
HLS uses the configured FFmpeg binary, `remux_timeout_ms`, and the
`[transcode]` hardware/concurrency policy. HLS artifacts are staged below
`remux_staging_root/hls`.
WebDAV source inputs are staged under `remux_staging_root/inputs` before HLS
planning.
Remote input staging is subject to the configured `[staging].max_bytes` disk
budget. Expired staged inputs are cleaned during startup when
`[staging].cleanup_on_startup` is enabled.
Remote direct-play response bodies are bounded by
`[playback].remote_stream_concurrency`; remote probe and FFmpeg input staging
are bounded by `[playback].remote_stage_concurrency`.
NFO import/export use the configured library VFS backend. Export requires a
writable backend; read-only remote backends return `400 unsupported`.
Segment lines are rewritten to session-scoped URLs:

```text
/playback/sessions/{session_id}/hls/segments/{segment_name}
```

`GET /playback/sessions/{session_id}/hls/segments/{segment_name}` serves a
generated HLS segment for a finished HLS session. Segment names are constrained
to the session output directory. Missing segments return `404 not_found`.
Segments requested for a non-HLS session return `400 invalid_input`. Segments
requested before the HLS session reaches `finished` return `409 conflict`.

### Remote Storage And Playback Limitations

WebDAV remote storage is read-only. `[[libraries]]` can configure multiple
local and WebDAV libraries, and persisted playback sources resolve back to
their configured library backend. WebDAV scan/list/stat/open-range, probe
staging, direct range reads, direct-play response-body streaming, remux
staging, HLS staging, startup cleanup, and manifest-backed staging disk budget
are covered.

Remux and HLS still stage full remote objects before FFmpeg. Remote NFO sidecar
import works through VFS, but remote NFO export is rejected unless the backend
advertises writable capabilities. Remote-byte cache, S3-compatible storage, and
remote sidecar writes are not part of the current WebDAV backend.

## Webhook Routes

`POST /webhooks/endpoints` upserts a webhook endpoint. Request bodies use
secret references, not plaintext secrets:

```json
{
  "id": null,
  "name": "receiver",
  "url": "https://example.test/taru-webhook",
  "secret_env": "TARU_WEBHOOK_SECRET",
  "subscribed_event_kinds": ["library.scanned"],
  "timeout_ms": 5000,
  "max_attempts": 3,
  "status": "enabled"
}
```

`subscribed_event_kinds` accepts known event kind strings or `"*"`. The initial
event kinds are `library.scanned`, `item.metadata_refreshed`, `nfo.imported`,
`nfo.exported`, and `playback.session_finished`.

Webhook endpoint responses return the persisted endpoint record, including
`secret_env`, but never the resolved secret value.

`GET /webhooks/endpoints` lists enabled endpoints. `GET
/webhooks/endpoints/{endpoint_id}` returns one endpoint or `404 not_found`.

`POST /events/{event_id}/webhooks/deliver` explicitly dispatches one persisted
outbox event to enabled endpoints whose subscriptions match the event kind. It
returns the outbox event, delivery counters, created attempts, skipped endpoint
count, and safe per-endpoint errors when dispatch fails before an attempt can
be recorded.

Webhook delivery bodies use a versioned JSON envelope and include these
headers:

```text
content-type: application/json
x-taru-event-id: <event id>
x-taru-event-kind: <event kind>
x-taru-signature: sha256=<hmac hex>  # present when secret_env is configured
```

`GET /events/{event_id}/webhook-attempts` returns all recorded delivery
attempts for one event. Attempts include endpoint ID, attempt number, status,
HTTP status when available, safe error text, requested/completed timestamps,
and `next_retry_at` for retryable failures.

## Addon Routes

`POST /addons` registers or updates an HTTP addon manifest. Addons are disabled
by default. A caller must explicitly request `"status": "enabled"` and grant
every scope required by each declared resource before an enabled registration is
accepted.

```json
{
  "id": null,
  "manifest": {
    "id": "example.metadata",
    "name": "Example Metadata",
    "version": "0.1.0",
    "protocol_version": "2026-05-15",
    "base_url": "https://example.test/addon",
    "description": "Metadata suggestion addon",
    "resources": [
      {
        "kind": "metadata",
        "path": "/metadata",
        "input_schema": "taru.metadata.request.v1",
        "output_schema": "taru.metadata.response.v1",
        "required_scopes": ["item_metadata_read", "item_metadata_suggest"],
        "timeout_ms": 5000,
        "max_attempts": 2
      }
    ],
    "auth": "bearer",
    "default_timeout_ms": 10000,
    "default_max_attempts": 2,
    "scopes": ["item_metadata_read", "item_metadata_suggest"]
  },
  "granted_scopes": ["item_metadata_read", "item_metadata_suggest"],
  "status": "disabled"
}
```

The current addon protocol version is `2026-05-15`. Taru rejects manifests
with unsupported protocol versions, non-HTTP base URLs, relative resource
paths, duplicate resource declarations, invalid timeout/retry bounds, or
resource scopes that are not declared by the manifest.

Persisted registration responses include the manifest snapshot,
`granted_scopes`, and enabled/disabled status. They do not include resolved
runtime secrets.

`GET /addons` lists registrations. `GET /addons?status=enabled` and
`GET /addons?status=disabled` filter by status. `GET /addons/{addon_id}`
returns one registration or `404 not_found`.

Addon resource calls use a versioned request/response envelope in
`taru-addon-protocol`. Calls are bounded by timeout and `max_attempts`; 408,
429, 5xx, and transport errors are retryable, while other 4xx responses fail
without retry. HTTP handlers only register and inspect addons in M5; they do
not call addon resource endpoints inline.

The workspace includes `taru-reference-addon`, a minimal metadata addon fixture
used by the M5.5 end-to-end test. It proves that a local addon can be
registered, queried, and called through the protocol transport.

## Automation Routes

`POST /automation/providers` upserts an external automation provider
configuration. Provider credentials are secret references:

```json
{
  "id": null,
  "name": "gateway",
  "base_url": "https://example.test/automation",
  "secret_env": "TARU_AUTOMATION_SECRET",
  "capabilities": ["summary", "recommendation"],
  "timeout_ms": 30000,
  "max_attempts": 2,
  "status": "enabled"
}
```

Initial capabilities are `recommendation`, `metadata_cleanup`, `summary`, and
`title_match`.

`POST /automation/jobs` enqueues an automation job and returns `202 Accepted`
with the normal job envelope. The request stores a prompt snapshot and does not
store the resolved provider secret:

```json
{
  "provider_id": "018f0000-0000-7000-8000-000000000001",
  "capability": "summary",
  "library_id": null,
  "item_id": null,
  "source_id": null,
  "prompt": {
    "title": "The Matrix"
  },
  "idempotency_key": "summary:matrix"
}
```

Generated results are stored as automation artifacts with status `proposed`.
They are not canonical metadata until a future explicit acceptance/writeback
policy is implemented.

`GET /automation/jobs/{job_id}/artifacts` lists artifacts created by one job.
`GET /items/{item_id}/automation/artifacts` lists artifacts associated with an
item.

## Playback Error Summary

Playback routes use the same error envelope as the rest of the API.

```text
unknown source/session/item       -> 404 not_found
invalid client capability query   -> 400 invalid_input
unsupported local playback source -> 400 unsupported
in-flight equivalent remux/HLS    -> 409 conflict
unfinished HLS segment session    -> 409 conflict
storage read/metadata failures    -> 502 storage_error/provider_error
storage timeout                   -> 504 storage_timeout
storage unauthorized/forbidden    -> 502 storage_unauthorized
storage rate-limited              -> 503 storage_rate_limited
staging budget exhausted          -> 507 staging_budget_exhausted
staging validation mismatch       -> 502 staging_validation_mismatch
FFmpeg runner failure             -> 502 ffmpeg_error
database failures                 -> 500 database_error
```
