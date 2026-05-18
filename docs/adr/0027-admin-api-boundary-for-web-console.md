# 0027: Define a Versioned Admin API Boundary for the Web Console

Status: accepted

## Context

ADR 0023 stabilizes the Public Client API error/version contract. ADR 0025
keeps generated Public Client OpenAPI and SDK surfaces limited to
protocol-owned client routes. ADR 0026 positions web as an administration,
setup, remote-control, and light browsing surface rather than the flagship
playback client.

The admin web console workstream needs a contract decision before UI generation
or implementation. Current Taru routes already expose many admin-oriented
operations, but they sit beside public client routes:

- scan and NFO import/export jobs;
- ingestion failure diagnostics;
- metadata attempts, raw cache, provider status, and maintenance planning;
- storage backend diagnostics;
- job detail;
- webhook, automation, and addon administration;
- playback session inspection and cancellation through the Public Client API.

The risk is not that these routes exist. The risk is that future web console
work expands the Public Client API or `taru-client-protocol` with admin-only
diagnostics because those DTOs are convenient for a browser UI.

## Decision

Taru will define a separate **Admin API** boundary for the web console.

Admin routes that are not part of the Public Client API should live under a
versioned admin namespace:

```text
/admin/v1/*
```

The namespace is a contract boundary, not a requirement to duplicate every
existing route immediately. Existing pre-namespace admin/internal routes may
remain while the first console and route migration are planned, but new
admin-only surfaces should use `/admin/v1/*` unless a later ADR supersedes
this decision.

Public client routes remain the stable client contract described by ADR 0023
and ADR 0025. The admin console may read Public Client API routes when the
information is genuinely client-facing, such as library list, item browse,
source probe, playback decisions, or playback session detail. It must not make
admin-only diagnostics public merely because the web console needs them.

## DTO Ownership

Admin DTOs belong in the AGPL server adapter boundary:

- `taru-api::admin` for jobs, storage diagnostics, ingestion failures, startup
  or server diagnostics, and future admin overview DTOs;
- `taru-api::metadata_diagnostics` for provider attempts, raw cache, provider
  runtime, maintenance plans, and cleanup DTOs;
- `taru-api::extension` for webhook, automation, and addon administration;
- future focused `taru-api` modules when a surface grows large enough, such as
  `admin_catalog`, `admin_playback`, or `admin_settings`.

Admin DTOs must not move into `taru-client-protocol` unless they become
genuine Public Client API concepts. Public client protocol crates remain
permissive, dependency-light, and free of server/admin internals.

## Versioning And Compatibility

`/admin/v1/*` is versioned independently from the Public Client API:

- admin v1 may evolve faster than public client v1;
- breaking admin changes require either a documented migration in the
  admin-web-console workstream or a future admin API ADR;
- public `/health` and `x-taru-api-version` continue to describe the Public
  Client API version, not the whole Admin API;
- an admin overview or capability endpoint may later report admin API version
  and feature availability.

Admin routes should use the same baseline error envelope shape as Public Client
API routes so UI error handling stays predictable:

```json
{
  "code": "not_found",
  "message": "not found: job 018f..."
}
```

Admin error codes may include admin-only values, but they must remain stable
enough for UI branching within an accepted admin version.

## Leakage And Redaction Rules

Admin API responses are operationally richer than Public Client API responses,
but they are not raw internal dumps.

Admin routes must not expose:

- plaintext secrets, bearer tokens, API keys, webhook signing secrets, addon
  tokens, or resolved provider credentials;
- authorization headers or raw request headers containing secrets;
- raw local filesystem paths unless a route explicitly documents why an
  admin-only path is required and how it is redacted or scoped;
- server process internals that would make durable UI state depend on
  implementation details;
- raw provider response bodies outside explicit diagnostics routes;
- addon-hosted pages as if they were trusted first-party admin UI.

Admin routes should prefer:

- secret references such as environment variable names;
- typed failure classes and stable status strings;
- redacted local path or backend capability summaries;
- job IDs, item IDs, source IDs, library IDs, event IDs, and session IDs;
- safe summaries before raw diagnostics;
- explicit route-level docs when a response can be sensitive.

## Route Migration Direction

The first Admin API implementation slices should not try to move every route
at once. The accepted sequence is:

1. Keep the current route matrix as an inventory.
2. Add new missing admin-only surfaces under `/admin/v1/*`.
3. When touching existing admin/internal routes for console work, either expose
   the new `/admin/v1/*` shape or add a compatibility wrapper.
4. Keep Public Client OpenAPI and SDK inventory tests rejecting admin surfaces.
5. Generate any future Admin API contract separately from Public Client
   OpenAPI/SDK artifacts.

The recommended first admin slices are:

- overview support: job/session/event list or a read-only overview summary;
- playback diagnostics: hardware capability, policy, FFmpeg status, runtime
  budgets, and staging summary without unsafe local paths;
- catalog governance: unknown items, duplicate-source relationships, provider
  mappings, local inference evidence, and NFO sidecar status before repair
  mutations;
- extension operations: complete webhook, automation, and addon list/lifecycle
  semantics.

## Consequences

- The web admin console can grow without weakening the Public Client API
  boundary.
- `taru-client-protocol` stays suitable for permissive mobile, web, CLI, and
  SDK consumers.
- `taru-api` remains the right home for admin/server DTOs, but it may need
  more focused modules as Admin API breadth grows.
- Future UI generation can mock admin data without pretending that every
  product page has an existing stable HTTP route.
- Admin API route tests should cover redaction and public route inventory
  separation when implementation begins.

## Alternatives Considered

- Use existing root-level routes without a namespace. Rejected because the
  boundary between public and admin surfaces would stay ambiguous as the
  console grows.
- Put admin DTOs into `taru-client-protocol`. Rejected because admin
  diagnostics are server-owned, AGPL-adjacent, and not stable client concepts.
- Generate one combined OpenAPI contract for public and admin routes. Rejected
  for the first admin slice because public SDK generation must continue to
  reject admin/internal surfaces.
- Choose `/api/admin/*` without an admin version. Rejected because the console
  will likely need faster iteration than the Public Client API and should have
  an explicit compatibility boundary.
- Build the UI first and infer the Admin API later. Rejected because it would
  encourage mock data and generated components to become accidental contracts.

## Related Workstreams

- `docs/workstreams/admin-web-console/`
- `docs/workstreams/public-api-contract/`
- `docs/workstreams/openapi-client-contract/`
- `docs/workstreams/client-sdk-contract/`
