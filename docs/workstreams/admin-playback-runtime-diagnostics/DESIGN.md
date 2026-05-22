# Admin Playback Runtime Diagnostics Design

Status: Completed
Last updated: 2026-05-18

## Problem

The admin web console can now list playback sessions through M55, but it still
cannot explain the runtime context behind those sessions:

- which hardware acceleration policy was configured;
- which acceleration mode Nako selected after FFmpeg capability probing;
- whether fallback to CPU was used;
- which hardware capabilities FFmpeg reported;
- what transcode and remote playback budgets are configured;
- whether staging cleanup is enabled and what startup cleanup did.

Operators need this information to diagnose failed or slow playback without
reading local config files, server logs, process internals, or filesystem
paths.

## Target State

Add a read-only Admin API v1 route:

```text
GET /admin/v1/playback/runtime
```

The response should include safe summaries for:

- Admin and Public Client API versions;
- FFmpeg runtime capability status without exposing `ffmpeg_path`;
- configured hardware acceleration policy;
- selected HLS transcode acceleration and fallback reason;
- FFmpeg hardware capability report;
- transcode CPU/GPU slot budgets and the selected HLS budget;
- remux timeout and configured remux concurrency;
- remote stream/stage permit budgets already exposed by storage diagnostics;
- staging cleanup configuration and startup cleanup counts.

## Scope

In scope:

- `nako-api::admin` DTOs for playback runtime diagnostics;
- `nako-server::app::playback` snapshot support;
- `nako-server::http::admin` route wiring;
- focused API/server tests;
- public OpenAPI/SDK leakage checks;
- admin-web-console and goal/workstream docs.

Out of scope:

- no Public Client API route or DTO changes;
- no `nako-client-protocol` changes;
- no admin OpenAPI generation;
- no playback session mutation;
- no playback source selection deepening;
- no adaptive HLS ladder;
- no FFmpeg runner behavior changes;
- no hardware probing strategy change beyond exposing existing evidence;
- no frontend UI implementation.

## Architecture Direction

The Admin API response is a safe operational view, not a raw dump.

`nako-transcode` remains the owner of hardware report, selection, and resource
budget concepts. `nako-server::app::playback` should expose a small immutable
diagnostics snapshot derived from current configuration and the HLS runtime's
selected acceleration. `nako-server::http::admin` translates that snapshot into
admin-owned DTOs.

The route must not expose:

- `ffmpeg_path`;
- `remux_staging_root`;
- HLS output paths;
- transcode session `output_path`;
- local library roots;
- raw process handles, cancellation tokens, or internal semaphores;
- secrets, tokens, or provider raw responses.

The first slice can expose configured budget values rather than every live
permit count. Storage backend diagnostics already carry per-library remote
stream/stage permit availability, and the Admin playback runtime route can
include an aggregate summary derived from that safe diagnostic shape.

## Follow-Ons

- Admin playback session detail route if list rows need richer per-session
  runtime evidence.
- Deeper **Playback Source Selection** diagnostics after the selection model is
  expanded for subtitles, HDR, audio tracks, client profiles, bandwidth, and
  Source Variants.
- Admin OpenAPI generation once the Admin API has enough stable breadth.
