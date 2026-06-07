# Admin Jobs Queue Pressure Diagnostics

## Problem

Nako durable job storage already has queue-pressure aggregation: counts grouped
by kind, status, and resource class, plus claimable, delayed retry, oldest
queued, and next retry timing. Operators currently need to infer pressure from
individual Admin Jobs rows, which makes backlog and delayed retry health hard to
scan.

Jellyfin's Scheduled Tasks surface exposes compact lifecycle summaries for
operator visibility and pushes task changes over realtime channels. Nako should
not copy Jellyfin's scheduled-task framework because the control-plane baseline
is durable jobs with resource classes and redacted diagnostics. The right next
step is to expose Nako's existing queue-pressure read model through the Admin
Jobs response and render it in Admin Web.

## Scope

- Add a redaction-safe `AdminJobQueuePressureSummary` DTO in the Admin API.
- Extend `AdminJobListResponse` with `queue_pressure`.
- Map existing `JobQueuePressureSummary` values from the server app service into
  the Admin Jobs list response.
- Regenerate Admin Web TypeScript contracts from `nako-api`.
- Render a compact Jobs queue-pressure section in Admin Web using generated
  contract types and deterministic mock fallback data.
- Add focused API, server, and Admin Web tests for serialization, redaction,
  generated contract drift, and rendering.

## Non-Goals

- Do not add a new scheduled-task framework.
- Do not add WebSocket or realtime Admin Jobs updates in this slice.
- Do not change job claiming, scheduling, retry, or priority behavior.
- Do not add a new database query; reuse the existing queue-pressure repository
  contract exposed through `NakoApp`.
- Do not expose raw durable `input_json`, `summary_json`, raw `error`, storage
  URIs, local paths, backend URLs, credentials, etags, fingerprints, URI
  digests, source locators, or cache payloads.

## Acceptance Criteria

- `GET /admin/v1/jobs` returns `queue_pressure` with grouped safe queue facts.
- Queue-pressure values include kind, status, resource class, total count,
  claimable count, delayed retry count, oldest queued timestamp, and next retry
  timestamp.
- Admin Web Jobs renders the queue-pressure summary above or near the jobs
  table without client-side recomputation from row data.
- Generated Admin contracts under `apps/admin-web` and `web` are refreshed from
  `nako-api`.
- API serialization tests prove the new DTO shape and reject raw durable payload
  field names.
- Server route tests prove the Admin Jobs response maps persisted queue pressure
  and stays redaction-safe.
- Admin Web tests prove mock/live rendering, Chinese copy, and redaction.
- Focused Rust and Admin Web validation passes or any unavailable gate is
  recorded with the failure reason.
