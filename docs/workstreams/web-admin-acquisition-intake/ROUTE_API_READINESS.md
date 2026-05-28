# Web Admin Acquisition Intake - Route/API Readiness

Status: Active
Last updated: 2026-05-28

## Route Target

| Frontend route | Surface | Status | Notes |
| --- | --- | --- | --- |
| `/admin/acquisition/intake` | Admin | Planned | New `web/` route with route-owned query state and read-only candidate diagnostics. |

## Generated Admin Contracts

The committed generated Admin contract already includes the acquisition intake
paths and DTOs needed for the first read-only route:

| Contract | Generated symbol | Path or shape | First use |
| --- | --- | --- | --- |
| Candidate list route | `ADMIN_API_ROUTES.acquisitionIntakeCandidates` | `/admin/v1/acquisition/intake/candidates` | Live candidate list data source. |
| Candidate query | `AdminAcquisitionIntakeCandidatesQuery` | `library_id`, `state`, `source_kind`, `managed_import_artifact_id`, `limit`, `offset` | Route search-param mapping. |
| Candidate list response | `AdminAcquisitionIntakeCandidateListResponse` | `admin_api_version`, `public_api_version`, `candidates`, `page` | Data-source contract test and read model mapping. |
| Candidate diagnostic | `AdminAcquisitionIntakeCandidateDiagnostic` | Redacted source fields, state, Managed Import artifact id, timestamps | UI table/cards. |
| Watch-folder discovery | `ADMIN_API_ROUTES.acquisitionIntakeWatchFolderDiscovery` and `AdminWatchFolderDiscoveryRequest` | mutation request/response | Deferred until mutation guards are explicit. |

## First Read-Only Mapping

`WAAI-020` should verify and document the final field mapping before code work,
but the initial mapping is:

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

## Required Gates

```bash
npm --prefix web run test -- src/test/data-source-contracts.test.ts
npm --prefix web run test -- src/test/route-contracts.test.tsx src/test/route-state-contracts.test.tsx
npm --prefix web run check
npm --prefix web run build:budget
```

Closeout should also include browser smoke for desktop and mobile viewports.
