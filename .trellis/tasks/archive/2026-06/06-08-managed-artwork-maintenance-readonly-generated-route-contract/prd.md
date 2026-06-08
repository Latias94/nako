# Managed Artwork Maintenance Readonly Generated Route Contract

## Goal

Make the existing read-only Managed Artwork maintenance diagnostics reachable
through the generated Admin API contract and a bounded Admin Web operator page.
This continues the overnight Jellyfin comparison campaign by closing another
hidden Admin route gap without adding destructive cleanup behavior.

## What I Already Know

- The parent campaign is comparing Nako with `repo-ref/jellyfin` and shipping
  independently verified fearless-refactor slices.
- Jellyfin exposes image/cache and system storage diagnostics as operator
  surfaces, but Nako's equivalent must stay in Nako terms: **Managed Artwork**,
  **Selected Artwork**, and **Nako-Managed Artifact**.
- Nako already has server handlers and redaction-safe DTOs for:
  - `GET /admin/v1/artwork/artifacts/lifecycle`
  - `GET /admin/v1/artwork/artifacts/storage-drift`
  - `GET /admin/v1/artwork/artifacts/remediation-plan`
- These three read-only routes are currently explicit exclusions in
  `crates/nako-api/src/admin_contract.rs`.
- Destructive or mutating artwork maintenance routes remain excluded in this
  slice:
  - candidate accept,
  - ingest process/requeue,
  - artifact publish,
  - cleanup,
  - remediate stray files.
- Existing API tests already prove Managed Artwork diagnostic DTOs do not expose
  `storage_uri`, `managed-artwork://...`, local paths, artifact root,
  `source_uri`, `cache_uri`, raw source URLs, provider query strings, token
  material, or content hashes.

## Reference-Code Boundary

- Jellyfin is reference material only. Do not copy, translate, or import
  Jellyfin code, comments, schemas, tests, or assets.
- Use Jellyfin to validate the operator workflow idea: artwork/cache storage
  pressure should be diagnosable.
- Implement original Nako behavior against Nako DTOs, specs, and Admin Web
  route patterns.

## Requirements

- Add generated Admin route keys for the three read-only diagnostics:
  - `managedArtworkArtifactLifecycle`
  - `managedArtworkArtifactStorageDrift`
  - `managedArtworkArtifactRemediationPlan`
- Remove those three routes from `ADMIN_ROUTE_EXCLUSION_SUFFIXES`.
- Add the missing TypeScript DTOs to the generated Admin contract body for:
  - lifecycle response, summary, and item;
  - storage drift response, summary, missing artifact, stray file, and enums;
  - remediation plan response, summary, missing artifact, stray file, and enums.
- Regenerate both Admin TypeScript contract copies:
  - `apps/admin-web/src/adminApi/generated/contract.ts`
  - `web/src/api/admin/generated/contract.ts`
- Add typed Admin Web client/data-source methods for these diagnostics using
  generated route constants and query parameters.
- Add an Admin Web `/artwork/maintenance` read-only operator page with URL-owned
  `limit`, `offset`, `cleanup_candidates_only`, and `file_scan_limit` search
  params.
- The page must render only counts, booleans, safe enum codes, IDs, dimensions,
  byte counts, media type, and timestamps.
- The page must not render raw artifact file names, local paths, storage URI,
  `managed-artwork://` handles, source/cache URI, raw source URLs, provider query
  strings, token material, artifact root, or content hashes.
- Mock fallback may show safe read rows. No mock mutation success is allowed,
  because this slice has no mutations.

## Acceptance Criteria

- [ ] `nako-api` generated route inventory includes the three read-only Managed
      Artwork maintenance route constants.
- [ ] The mutating Managed Artwork maintenance routes remain explicit
      exclusions.
- [ ] Generated contract drift tests pass.
- [ ] Admin Web client tests cover generated routes and query params.
- [ ] Admin Web data-source tests cover safe mapping, read fallback, and
      redaction.
- [ ] Admin Web route tests cover `/artwork/maintenance`, URL-owned filters,
      zh-Hans copy, read fallback, and redaction.
- [ ] Focused Rust and Admin Web gates pass before commit.

## Definition Of Done

- Code and generated artifacts are updated.
- Task evidence records commands run and results.
- Relevant spec memory is updated if this establishes a reusable pattern.
- Commit only this slice with a Conventional Commit message.

## Out Of Scope

- Accepting artwork candidates.
- Processing or requeueing artwork ingests.
- Publishing artifacts.
- Cleaning artifact rows or deleting stray files.
- New durable artwork jobs, schema migrations, or automatic cleanup workers.

## Technical Notes

- Server route evidence: `crates/nako-server/src/http/admin.rs`.
- API DTO evidence: `crates/nako-api/src/admin/managed_artwork.rs`.
- Generated contract source: `crates/nako-api/src/admin_contract.rs`.
- Admin Web should follow the Access Invitation, Addon Task Run, Addon Event
  Delivery, and Item Artwork Gallery route/data-source/test patterns for safe
  projection and URL-owned filters.
