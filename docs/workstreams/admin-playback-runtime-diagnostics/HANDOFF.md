# Admin Playback Runtime Diagnostics Handoff

Status: Completed
Last updated: 2026-05-18

## Current State

M56 is complete. It added a safe read-only Admin API v1 route:

```text
GET /admin/v1/playback/runtime
```

## Completed Implementation

- admin-owned playback runtime diagnostics DTOs in `taru-api::admin`;
- playback app diagnostics snapshot support in `taru-server`;
- `GET /admin/v1/playback/runtime`;
- route tests for diagnostics shape, redaction, and auth protection;
- public OpenAPI/SDK leakage checks.

## Next Recommended Task

Open a follow-on for event outbox list/filter or storage staging/cache
diagnostics before adding broader admin console drill-down tables.

## Constraints

- Keep `TranscodeSessionResponse` and Public Client API unchanged.
- Keep Admin API DTOs in `taru-api::admin`.
- Do not expose `ffmpeg_path`, `remux_staging_root`, output paths, local library
  roots, secrets, tokens, runner handles, or cancellation tokens.
- Do not add playback mutations or runtime behavior in this slice.
- Keep `taru-client-protocol` unchanged.
