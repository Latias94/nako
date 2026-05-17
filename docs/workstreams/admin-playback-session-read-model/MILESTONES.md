# Admin Playback Session Read Model Milestones

Status: Completed
Last updated: 2026-05-18

## M55: Admin Playback Session List Read Model

Objective:

- Add a safe Admin API v1 read model for playback session lists.
- Preserve existing Public Client API playback session detail/cancel behavior.
- Keep local transcode output paths and runtime internals out of admin list
  responses.

Deliverables:

- `TranscodeSessionListFilter` or equivalent repository read model.
- SQLite list/filter implementation and focused tests.
- `AdminPlaybackSessionListResponse` and redacted list item DTOs.
- `GET /admin/v1/playback/sessions`.
- Focused HTTP tests and closeout docs.

Exit criteria:

- Admin Console can list/filter playback sessions through
  `/admin/v1/playback/sessions`.
- The list response does not expose `output_path`, local staging roots,
  filesystem paths, or process-local runtime internals.
- Existing Public Client API routes remain compatible.
- Public OpenAPI and SDK leakage checks still reject admin/internal surfaces.
- `crates/taru-client-protocol` remains unchanged.
- Validation gates passed on 2026-05-18.

## Follow-Ons

- Playback runtime diagnostics for hardware, FFmpeg, budgets, and staging.
- Event outbox list/filter for Automation/Webhooks.
- Admin playback session detail route if the console needs richer diagnostics.
