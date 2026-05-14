# Taru HTTP API

## Scope

This document describes the Phase 2.1 API discipline. It is not a complete
OpenAPI specification yet, but it fixes response shapes, pagination rules, job
envelopes, and error envelopes for current server routes.

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
unsupported
provider_error
storage_error
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
GET  /search?q=matrix&facet=genre:sci-fi&limit=50&offset=0
POST /items/{item_id}/metadata/refresh
GET  /sources/{source_id}/probe
GET  /jobs/{job_id}
```

`POST /libraries/{library_id}/scan` returns `202 Accepted` with a queued job.
The job runs in the background.

`POST /items/{item_id}/metadata/refresh` returns `202 Accepted` with a queued
metadata refresh job. The current implementation uses the library metadata
profile provider order and records provider attempts in `GET /jobs/{job_id}`.

`POST /libraries/{library_id}/nfo/import` and
`POST /libraries/{library_id}/nfo/export` return `202 Accepted` with queued NFO
jobs. Import reads same-stem `.nfo` files according to the library local
metadata policy. Export writes sidecars only when the policy is
`write_sidecar`.

`GET /search` reads the SQLite search projection behind `taru-search`. `facet`
is optional and comma-separated for the current lightweight route shape. Search
results return projected media items and relevance scores; richer filters and
ranking can be added behind the same search boundary.
