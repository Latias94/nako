# Taru HTTP API

## Scope

This document describes the current server API contract for the local
video-library playback MVP. It is not a complete OpenAPI specification yet,
but it fixes response shapes, pagination rules, job envelopes, playback
session envelopes, and error envelopes for current server routes.

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
GET  /sources/{source_id}/probe
GET  /sources/{source_id}/playback/decision
GET  /sources/{source_id}/stream
HEAD /sources/{source_id}/stream
GET  /sources/{source_id}/stream/remux
GET  /sources/{source_id}/stream/hls/playlist.m3u8
GET  /playback/sessions/{session_id}
GET  /playback/sessions/{session_id}/hls/segments/{segment_name}
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

`GET /sources/{source_id}/stream` serves direct play bytes for local sources.
It supports HTTP `Range` requests and returns `206 Partial Content` with
`Accept-Ranges`, `Content-Range`, and `Content-Length` when a satisfiable range
is requested.

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
request completes or after a server restart.

`GET /playback/sessions/{session_id}` returns the persisted session state for
remux and HLS transcode sessions. The response includes the source ID, session
kind, request key, staged output path, state, failure category/message, and
lifecycle timestamps. Missing sessions return `404 not_found`.

`GET /sources/{source_id}/stream/hls/playlist.m3u8` starts or reuses a minimal
single-variant HLS transcode session and returns a rewritten media playlist.
HLS uses the configured FFmpeg binary, `remux_timeout_ms`, and the
`[transcode]` hardware/concurrency policy. HLS artifacts are staged below
`remux_staging_root/hls`.
Segment lines are rewritten to session-scoped URLs:

```text
/playback/sessions/{session_id}/hls/segments/{segment_name}
```

`GET /playback/sessions/{session_id}/hls/segments/{segment_name}` serves a
generated HLS segment for a finished HLS session. Segment names are constrained
to the session output directory. Missing segments return `404 not_found`.
Segments requested for a non-HLS session return `400 invalid_input`. Segments
requested before the HLS session reaches `finished` return `409 conflict`.

## Playback Error Summary

Playback routes use the same error envelope as the rest of the API.

```text
unknown source/session/item       -> 404 not_found
invalid client capability query   -> 400 invalid_input
unsupported local playback source -> 400 unsupported
in-flight equivalent remux/HLS    -> 409 conflict
unfinished HLS segment session    -> 409 conflict
storage read/metadata failures    -> 502 storage_error/provider_error
database failures                 -> 500 database_error
```
