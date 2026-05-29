# Web Admin Acquisition Intake - Route/API Readiness

Status: Route implemented
Last updated: 2026-05-29

## Route Target

| Frontend route | Surface | Status | Notes |
| --- | --- | --- | --- |
| `/admin/acquisition/intake` | Admin | Implemented | New `web/` route with route-owned query state and read-only candidate diagnostics. WAAI-020 added the data-source/read-model boundary; WAAI-030 wired the route, page, filters, pagination, and redaction tests. |

## Generated Admin Contracts

The committed generated Admin contract includes the acquisition intake paths and
DTOs needed for the first read-only route. WAAI-020 verified these generated
symbols in `web/src/api/admin/generated/contract.ts` and wired the read-only
candidate list into the Admin data-source boundary:

| Contract | Generated symbol | Path or shape | First use |
| --- | --- | --- | --- |
| Candidate list route | `ADMIN_API_ROUTES.acquisitionIntakeCandidates` | `/admin/v1/acquisition/intake/candidates` | Live candidate list data source via `AdminApiClient.getAcquisitionIntakeCandidates`. |
| Candidate query | `AdminAcquisitionIntakeCandidatesQuery` | `library_id`, `state`, `source_kind`, `managed_import_artifact_id`, `limit`, `offset` | Route search-param mapping; data source normalizes blank strings away and defaults to `limit=50&offset=0`. |
| Candidate list response | `AdminAcquisitionIntakeCandidateListResponse` | `admin_api_version`, `public_api_version`, `candidates`, `page` | Mapped to `AdminAcquisitionIntakeReadModel`. |
| Candidate diagnostic | `AdminAcquisitionIntakeCandidateDiagnostic` | Redacted source fields, state, Managed Import artifact id, timestamps | Mapped to `AdminAcquisitionIntakeCandidateReadModel`; unknown raw fields are ignored. |
| Watch-folder discovery | `ADMIN_API_ROUTES.acquisitionIntakeWatchFolderDiscovery` and `AdminWatchFolderDiscoveryRequest` | mutation request/response | Deferred until mutation guards are explicit. |

## First Read-Only Mapping

`WAAI-020` verified and implemented the read-model mapping:

| UI field | Contract field |
| --- | --- |
| Candidate id | `id` |
| Target library | `target_library_id` |
| Source kind | `source_kind` plus `custom_source_kind` |
| Source summary | `source_scheme` and `source_ref_redacted` |
| Fingerprint | `source_key_fingerprint` |
| Size | `size_bytes` |
| Managed Import linkage | `managed_import_artifact_id` |
| State/readiness | `state`, `has_diagnostics`, `has_intended_locator`, `has_fingerprint` |
| Timestamps | `first_seen_at_ms`, `last_seen_at_ms`, `created_at_ms`, `updated_at_ms` |

## Web Read-Model Boundary

WAAI-020 added the boundary but not the page:

| Web symbol | Role |
| --- | --- |
| `AdminApiClient.getAcquisitionIntakeCandidates(query)` | Live Admin API call using the generated route and query DTO. |
| `createAdminReadModelsDataSource().loadAcquisitionIntake(query)` | Route-facing read model entry point with live/fixture fallback. |
| `ADMIN_ACQUISITION_INTAKE_READ_MODEL_FIXTURE` | Explicit fixture for local development and tests. |
| `AdminAcquisitionIntakeReadModel` | UI-safe envelope with versions, normalized query, candidates, and page info. |
| `AdminAcquisitionIntakeCandidateReadModel` | Redacted candidate row/card model. |

`/admin/acquisition/intake` uses the data-source method above. The route owns
browser search params and serializes only:

```text
library_id
state
source_kind
managed_import_artifact_id
limit
offset
```

Default route state should match the data source defaults: `limit=50`,
`offset=0`, and no optional filters.

## Redaction Assertions

The read model intentionally exposes `sourceSummary` from
`source_ref_redacted`. It does not expose:

- raw intended locators;
- local filesystem paths;
- credentials or bearer tokens;
- prompt bodies;
- downloader internals;
- watch-folder mutation request bodies.

`src/test/data-source-contracts.test.ts` now covers live candidate mapping,
query serialization, Bearer authorization, fixture fallback, and ignoring
non-contract raw fields such as `intended_locator` or `prompt_body`.

## Deferred Mutation Boundary

Watch-folder discovery remains deferred to WAAI-040. No route, data source, or
UI behavior added by WAAI-020 performs acquisition mutations, promotion/apply,
or direct library writes.

## Required Gates

```bash
npm --prefix web run test -- src/test/data-source-contracts.test.ts
npm --prefix web run test -- src/test/route-contracts.test.tsx src/test/route-state-contracts.test.tsx
npm --prefix web run check
npm --prefix web run build:budget
```

Closeout should also include browser smoke for desktop and mobile viewports.

## WAAI-030 Route Evidence

WAAI-030 added `web/src/features/admin/admin-acquisition-intake.tsx`, route
wiring in `web/src/shell/nako-router.tsx`, and Admin navigation wiring in
`web/src/features/admin/admin-surface.tsx`.

The route is read-only. It calls `loadAcquisitionIntake(query)`, renders only
the redacted read-model fields, and keeps watch-folder discovery deferred to
WAAI-040.
