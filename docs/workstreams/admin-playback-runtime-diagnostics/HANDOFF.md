# Admin Playback Runtime Diagnostics Handoff

Status: Completed
Last updated: 2026-05-18

## Current State

M56 is complete. It added a safe read-only Admin API v1 route:

```text
GET /admin/v1/playback/runtime
```

## Completed Implementation

- admin-owned playback runtime diagnostics DTOs in `nako-api::admin`;
- playback app diagnostics snapshot support in `nako-server`;
- `GET /admin/v1/playback/runtime`;
- route tests for diagnostics shape, redaction, and auth protection;
- public OpenAPI/SDK leakage checks.

## Next Recommended Task

Open a follow-on for event outbox list/filter or storage staging/cache
diagnostics before adding broader admin console drill-down tables.

## Constraints

- Keep `TranscodeSessionResponse` and Public Client API unchanged.
- Keep Admin API DTOs in `nako-api::admin`.
- Do not expose `ffmpeg_path`, `remux_staging_root`, output paths, local library
  roots, secrets, tokens, runner handles, or cancellation tokens.
- Do not add playback mutations or runtime behavior in this slice.
- Keep `nako-client-protocol` unchanged.
