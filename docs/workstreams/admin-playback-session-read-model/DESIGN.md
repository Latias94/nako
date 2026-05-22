# Admin Playback Session Read Model Design

Status: Completed
Last updated: 2026-05-18

## Problem

ADR 0027 accepts `/admin/v1/*` for admin-only web console routes. M52 added
`GET /admin/v1/overview`; M54 added `GET /admin/v1/jobs`. Playback & Transcode
is still missing the equivalent read model for session lists.

Current Public Client API playback session support is intentionally narrow:

- `GET /playback/sessions/{session_id}` returns one known session.
- `POST /playback/sessions/{session_id}/cancel` requests cancellation.
- `GET /playback/sessions/{session_id}/hls/segments/{segment_name}` serves
  media output.

Those routes are useful to clients that already know a session ID. They are not
an admin operational list. The underlying `TranscodeSessionRecord` also
contains `output_path`, which must not leak into an admin list response.

## Target State

Add a safe Admin API v1 read model:

- `GET /admin/v1/playback/sessions`;
- filters for state, kind, Media Source, and pagination;
- admin-owned DTOs in `nako-api::admin`;
- repository list/filter support in `nako-core`/`nako-db`;
- a thin server app method and HTTP handler;
- tests proving route behavior, filtering, auth protection, and redaction.

## Scope

In scope:

- `crates/nako-core/src/repository/transcode.rs`
- `crates/nako-db/src/playback.rs`
- `crates/nako-api/src/admin.rs`
- `crates/nako-server/src/app/playback/mod.rs`
- `crates/nako-server/src/http/admin.rs`
- `crates/nako-server/src/http/query.rs`
- focused DB/API/server tests
- admin-web-console and goal evidence updates

Out of scope:

- no public client route changes;
- no `nako-client-protocol` changes;
- no public OpenAPI/TypeScript SDK expansion;
- no admin OpenAPI generation;
- no playback session mutation beyond existing known-ID cancel route;
- no runtime supervisor or transcode runner behavior changes;
- no UI implementation.

## Architecture Direction

Keep the Public Client API detail DTO and Admin API list DTO separate.
`TranscodeSessionResponse` remains client-facing and protocol-owned.
`AdminPlaybackSessionListItem` should expose operational state only:

- IDs;
- session kind/state;
- request key if it is already a stable request class;
- failure category and a safe failure-message presence flag or short message
  if proven safe;
- timestamps;
- flags such as `active` and `terminal`.

It must not expose:

- `output_path`;
- local staging roots;
- filesystem paths;
- internal runner handles;
- process-local cancellation tokens.

M55 applies this by exposing `has_failure_message` instead of raw failure
text and by omitting `output_path` entirely from the admin list item.

The repository owns filtering and pagination. The server handler owns query
parsing and response mapping, not SQL details.

## Follow-Ons

- Admin playback runtime diagnostics: hardware capability, FFmpeg status,
  resource budgets, and staging summary.
- Admin session detail route if the list needs richer diagnostics than the
  Public Client detail response.
- Session cancellation under `/admin/v1/*` only if the Admin API contract needs
  a versioned mutation wrapper.
