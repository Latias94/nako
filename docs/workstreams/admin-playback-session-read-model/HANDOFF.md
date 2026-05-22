# Admin Playback Session Read Model Handoff

Status: Completed
Last updated: 2026-05-18

## Current State

M55 is complete. The workstream added a safe Admin API v1 playback session
list/filter route.

Completed implementation:

- `TranscodeSessionListFilter` and SQLite list/filter support.
- `AdminPlaybackSessionListResponse` and redacted
  `AdminPlaybackSessionListItem`.
- `GET /admin/v1/playback/sessions`.
- HTTP tests for filtering, redaction, and auth protection.
- Admin-web-console docs now mark playback session list/filter as live.

## Next Recommended Task

Open a follow-on for playback runtime diagnostics:

- hardware acceleration report;
- FFmpeg availability;
- transcode resource budgets;
- staging budget and cleanup summary.

## Constraints

- Keep `TranscodeSessionResponse` and Public Client API unchanged.
- Keep Admin API DTOs in `nako-api::admin`.
- Do not expose `output_path` or local staging roots in admin list responses.
- Do not add playback mutations or runtime behavior in this slice.
- Keep `nako-client-protocol` unchanged.
