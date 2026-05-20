# Admin API TypeScript Contract Inventory

Status: Accepted
Last updated: 2026-05-19

This document records AATC-020: the current hand-written Admin API wire surface
in `apps/admin-web`, the corresponding `taru-api` DTO authority, and the first
generated artifact shape.

## Current Admin-Web Wire DTOs

`apps/admin-web/src/adminApi/types.ts` currently mixes two categories:

- Admin API wire DTOs copied from server responses.
- Admin-web local view models and source/fallback state.

Hand-written wire DTOs that should move to the generated contract:

| Type | Authority | Notes |
| --- | --- | --- |
| `PageInfo` | `taru_client_protocol::PageInfo` | Shared pagination wire shape used by public and admin responses. |
| `AdminOverviewResponse` | `taru-api::admin` | Includes `AdminOverviewStatus` as a string union. |
| `AdminCatalogGovernanceItemListResponse` | `taru-api::admin` | Includes local inference summaries and issue strings. |
| `AdminOutboxEventListResponse` | `taru-api::admin` | Uses a broad event subject shape in TS today. |
| `AdminJobListResponse` | `taru-api::admin` | Redacted list shape only; job detail is not in the first contract slice. |
| `AdminPlaybackSessionListResponse` | `taru-api::admin` | Redacted session list without output paths or failure messages. |
| `AdminPlaybackRuntimeDiagnosticsResponse` | `taru-api::admin` | Hardware policy/selection currently use broad object/string fields in TS. |
| `AdminStorageStagingDiagnosticsResponse` | `taru-api::admin` | Redacted staging/cache diagnostics without `source_uri` or `local_path`. |
| `AdminServerConfigDiagnosticsResponse` | `taru-api::admin` | Sanitized config diagnostics with secret references, not secret values. |

Local admin-web types that should remain outside generated contract:

- `DataSourceMode`
- `AdminSectionKey`
- `AdminSourceMap`
- `AdminErrorMap`
- `AdminConsoleData`
- `LibraryRow`
- `CatalogGovernanceSummary`
- `EventSummary`
- `JobRow`
- `PlaybackSummary`
- `StorageSummary`
- `SettingRow`

Those are view/data-source concepts, not HTTP contract types.

## Covered Route Inventory

The first generated contract should cover the routes already consumed by
`apps/admin-web/src/adminApi/client.ts`.

| Route key | Method and path | Response type | Server query support | First generated query type |
| --- | --- | --- | --- | --- |
| `overview` | `GET /admin/v1/overview` | `AdminOverviewResponse` | none | none |
| `catalogGovernanceItems` | `GET /admin/v1/catalog/governance/items` | `AdminCatalogGovernanceItemListResponse` | `library_id`, `max_confidence_milli`, `limit`, `offset` | `AdminCatalogGovernanceItemsQuery` |
| `events` | `GET /admin/v1/events` | `AdminOutboxEventListResponse` | `kind`, `status`, `library_id`, `source_id`, `limit`, `offset` | `AdminOutboxEventsQuery` |
| `jobs` | `GET /admin/v1/jobs` | `AdminJobListResponse` | `status`, `kind`, `resource_class`, `library_id`, `source_id`, `limit`, `offset` | `AdminJobsQuery` |
| `playbackSessions` | `GET /admin/v1/playback/sessions` | `AdminPlaybackSessionListResponse` | `source_id`, `kind`, `state`, `limit`, `offset` | `AdminPlaybackSessionsQuery` |
| `playbackRuntime` | `GET /admin/v1/playback/runtime` | `AdminPlaybackRuntimeDiagnosticsResponse` | none | none |
| `storageStaging` | `GET /admin/v1/storage/staging` | `AdminStorageStagingDiagnosticsResponse` | `purpose`, `state`, `limit`, `offset` | `AdminStorageStagingQuery` |
| `systemConfig` | `GET /admin/v1/system/config` | `AdminServerConfigDiagnosticsResponse` | none | none |

The query types are included because filters are the next admin-web UI slice.
They should be generated as optional string/number fields that match accepted
HTTP query names, while parsing and validation remain server-owned.

## Source Authority

Response DTO authority:

- `crates/taru-api/src/admin.rs`
- `taru_client_protocol::PageInfo` for pagination wire shape

Route behavior and query parsing currently live in:

- `crates/taru-server/src/http/admin.rs`
- `crates/taru-server/src/http/query.rs`

The first generated contract should not move server query parser structs into
`taru-client-protocol`. If query contracts need a Rust source later, add a
small Admin API contract-owned inventory in `taru-api`, not in the permissive
public protocol crate.

## Artifact Shape Decision

Chosen shape: **route constants + wire interfaces + query interfaces**.

Generate an app-local TypeScript file:

```text
apps/admin-web/src/adminApi/generated/contract.ts
```

The generated file should export:

- `TARU_ADMIN_API_VERSION`;
- `TARU_ADMIN_ROUTES`, keyed by stable route names;
- `AdminApiRouteKey`;
- `AdminPageQuery`;
- query interfaces for covered list routes;
- response interfaces and nested DTO types for covered routes.

Do not generate the fetch runtime in the first slice. Keep
`apps/admin-web/src/adminApi/client.ts` hand-written so base URL normalization,
bearer auth, request failure behavior, and future section-level fallback remain
owned by the app boundary.

## Alternatives Rejected

### Interfaces Only

Rejected because route paths would remain duplicated in `client.ts` and tests,
leaving route drift unsolved.

### Tiny Generated Client

Rejected for the first slice because it would force the generator to own
frontend-specific behavior: bearer-token storage policy, fetch injection,
error mapping, and future mock/live fallback ergonomics. Those are app
boundary decisions, not DTO contract decisions.

### Public SDK Reuse

Rejected by ADR 0025 and ADR 0027. Public Client SDK artifacts must continue
to reject `/admin/v1/*` routes and admin DTOs.

## AATC-030 Implementation Target

Recommended file scope:

- Add `crates/taru-api/src/admin_contract.rs` with
  `admin_typescript_contract()`.
- Add `crates/taru-api/examples/emit-admin-typescript-contract.rs`.
- Generate `apps/admin-web/src/adminApi/generated/contract.ts`.
- Update `apps/admin-web/src/adminApi/client.ts` to import route constants and
  generated response types.
- Keep UI view-model types in `apps/admin-web/src/adminApi/types.ts` or split
  them to a local `viewModels.ts` only if it reduces churn.

Required tests for AATC-030:

- generated app-local artifact matches the Rust generator output;
- generated Admin API route constants include the eight AWC-070 read routes;
- generated Admin API contract excludes known forbidden raw fields such as
  `source_uri`, `cache_uri`, `storage_uri`, `output_path`, `local_path`,
  `secret`, and token value terms;
- Public Client TypeScript SDK still excludes `/admin` and `/admin/v1`;
- admin-web `npm run check` passes after importing generated types.
