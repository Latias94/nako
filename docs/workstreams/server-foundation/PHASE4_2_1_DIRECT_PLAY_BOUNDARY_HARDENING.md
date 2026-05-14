# Phase 4.2.1: Direct Play Boundary Hardening

## Goal

Harden the Phase 4.2 direct play surface before adding remux, HLS, or
transcode sessions. The HTTP route should not own playback response policy.
It should translate request headers, call the application service, and stream
the file according to an explicit plan.

## Implemented Shape

### Streaming Response Plan

`taru-streaming` owns the reusable direct play response model:

- `DirectPlayRangeRequest`
- `DirectPlayResponseStatus`
- `DirectPlayResponsePlan`
- `plan_direct_play_response`

The plan resolves full responses, partial byte-range responses, malformed
range requests, and unsatisfiable ranges into one structure containing:

- status intent;
- content type;
- total source length;
- response body length;
- optional resolved byte range;
- optional `Content-Range`;
- file seek offset.

This keeps HTTP status and header policy out of the raw route body and gives
later remux/HLS work a clearer boundary to build on.

### Application Playback Plan

`taru-server::app` exposes `plan_direct_play`. It resolves the media source,
checks the current local VFS path hint requirement, reads source length, infers
content type, and attaches the streaming response plan.

The direct play route still serves local sources only. Remote staging and
byte-range cache behavior remain future work behind the VFS/cache boundary.

### HTTP Route Surface

The server exposes:

```text
GET  /sources/{source_id}/stream
HEAD /sources/{source_id}/stream
```

`GET` streams the planned body. `HEAD` returns the same direct play headers
without a response body, allowing clients to preflight source length, MIME
type, range support, and range validity.

Malformed, unsupported multi-range, and unsatisfiable range requests return
`416 Range Not Satisfiable` with `Content-Range: bytes */{total_len}` and
`Content-Length: 0`.

## Non-Goals

- No HLS output yet.
- No FFmpeg process orchestration yet.
- No remux session state yet.
- No hardware acceleration detection yet.
- No remote-source direct streaming or staging cache yet.

## Validation

Coverage added or updated for:

- direct play response planning for full, empty, partial, and invalid ranges;
- `HEAD /sources/{source_id}/stream` returning headers without a body;
- zero-byte direct stream responses;
- unsatisfiable ranges;
- unsupported multi-range requests;
- existing partial-content streaming behavior.

Required gates:

```text
cargo fmt --all -- --check
cargo check --workspace
cargo nextest run --workspace
git diff --check
```
