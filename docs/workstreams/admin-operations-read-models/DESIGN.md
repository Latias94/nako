# Admin Operations Read Models Design

Status: Completed
Last updated: 2026-05-18

## Why This Lane Exists

The admin web console now has overview, jobs, playback session, and playback
runtime read models. The next operational gaps are event history, storage
staging/cache state, and a safe view of the server's configured capabilities.

These are read-only operator diagnostics. They should explain what Nako is
doing without exposing local paths, event payloads, raw errors, resolved
secrets, provider raw responses, or process internals.

## Relevant Authority

- ADRs:
  - `docs/adr/0027-admin-api-boundary-for-web-console.md`
  - `docs/adr/0014-durable-event-outbox-for-webhooks-and-automation.md`
- Existing docs:
  - `CONTEXT.md`
  - `docs/api/HTTP_API.md`
  - `docs/workstreams/admin-web-console/ADMIN_API_MATRIX.md`
  - `docs/workstreams/admin-web-console/V0_CONTEXT.md`
- Related workstreams:
  - `docs/workstreams/durable-job-runtime-admin-read-model/`
  - `docs/workstreams/admin-playback-session-read-model/`
  - `docs/workstreams/admin-playback-runtime-diagnostics/`
  - `docs/workstreams/storage-vfs/`

## Problem

The server has durable operational records that are useful for administration,
but the admin web console cannot inspect them safely yet:

- event outbox records exist, but the only known-ID routes expose delivery
  attempts and manual dispatch;
- staging manifest records track remote staging/cache state, but there is no
  admin route that lists them without local paths;
- server config controls auth, libraries, metadata providers, transcode,
  staging, playback, and concurrency, but exposing the raw config would leak
  paths, URLs, and secret-bearing fields.

## Target State

Add three read-only Admin API v1 routes:

```text
GET /admin/v1/events
GET /admin/v1/storage/staging
GET /admin/v1/system/config
```

The responses should use admin-owned DTOs in `nako-api::admin`, backed by app
service methods in `nako-server`, and route-level tests that prove filtering,
pagination, auth protection, and redaction.

## In Scope

- Event outbox list/filter by kind, status, Media Library, Media Source, and
  pagination.
- Staging manifest diagnostics list/filter by purpose, state, and pagination.
- Staging/cache summary counters for configured budget, current manifest
  bytes, startup cleanup, and process-local backend cache count.
- Sanitized config diagnostics for auth status, configured library summaries,
  server concurrency, metadata runtime/provider settings, transcode policy,
  staging config, playback remote budgets, and webhook concurrency.
- Public OpenAPI/SDK leakage checks for new admin paths.
- Workstream, goal, HTTP API, and admin-web-console docs.

## Out Of Scope

- No event detail route and no raw event payload JSON.
- No webhook retry/cancel mutation beyond existing known-ID dispatch.
- No staging cleanup mutation.
- No raw VFS cache listing/object/failure list route unless a narrow repository
  port is added in a later slice.
- No raw `NakoServerConfig` serialization.
- No local library root, database URL, FFmpeg path, staging root, WebDAV base
  URL, WebDAV username, metadata provider proxy, literal provider header
  value, resolved token, or resolved password exposure.
- No Public Client API, public OpenAPI, generated SDK, Rust client SDK, or
  `nako-client-protocol` change.

## Starting Assumptions

| Assumption | Confidence | Evidence | Consequence if wrong |
| --- | --- | --- | --- |
| Admin routes belong in `nako-api::admin` and `nako-server::http::admin`. | High | ADR 0027 and M52-M56. | Keep DTO ownership narrow; split modules later if file size becomes the blocker. |
| Event outbox list/filter can extend the existing `EventOutboxRepository` without schema changes. | High | `event_outbox` already stores kind/status/library/source and has a paginated list. | Add a migration only if existing columns are insufficient. |
| Staging diagnostics can use existing manifest list and byte-sum APIs. | High | `StagingManifestRepository` already exposes list and `sum_staging_manifest_bytes`. | Add a narrow app method if route composition becomes too broad. |
| Sanitized config should summarize capability and secret-reference presence, not raw config. | High | Existing redaction rules in M52-M56 and `SecretString` behavior. | If operators need exact values, add separate explicit admin surfaces later. |

## Architecture Direction

The Admin API should remain a set of safe read models, not raw repository
records. Each route should translate from durable records or config into a
purpose-built DTO:

- event outbox list exposes IDs, kind, subject, status, counters, timestamps,
  and boolean payload/error presence;
- staging diagnostics expose source scheme, purpose, state, size, lease and
  expiry facts, and boolean validation-error presence;
- config diagnostics expose capabilities, counts, policies, secret-reference
  names, and enabled flags while withholding sensitive values and local paths.

`nako-server` owns composition because it can combine config, startup report,
storage diagnostics, and repository reads. `nako-api` owns DTO shape so the
wire contract remains explicit and testable.

## Closeout Condition

This lane can close when:

- all three routes exist and return redacted admin-owned DTOs;
- focused API/server/DB tests cover the redaction and filtering semantics;
- public OpenAPI and generated TypeScript SDK continue excluding admin paths;
- `crates/nako-client-protocol` has no diff;
- docs reflect the shipped behavior and remaining follow-ons are explicit.
