# Evidence

## Implementation Summary

- Added generated Admin route contract coverage for read-only Managed Artwork
  maintenance diagnostics:
  - `managedArtworkArtifactLifecycle`
  - `managedArtworkArtifactStorageDrift`
  - `managedArtworkArtifactRemediationPlan`
- Kept destructive or mutating Managed Artwork maintenance routes as explicit
  generated-contract exclusions.
- Added generated TypeScript DTO coverage for lifecycle, storage drift, and
  remediation plan responses, then refreshed both Admin contract consumers.
- Added Admin Web typed client and data-source methods for all three read-only
  routes.
- Added `/artwork/maintenance` Admin Web page with URL-owned `limit`, `offset`,
  `cleanup_candidates_only`, and `file_scan_limit` search params.
- Added route-local safe projection tests for counts, booleans, safe enum codes,
  IDs, dimensions, byte counts, media type, and timestamps only.
- Updated Trellis specs for the Managed Artwork Maintenance operator projection
  and read-only Admin contract pattern.

## Reference Comparison

- `research/jellyfin-managed-artwork-maintenance-comparison.md` records the
  Jellyfin comparison boundary.
- Jellyfin confirms the operator value of image/cache maintenance visibility.
- Nako implements the slice in Nako terms: Managed Artwork lifecycle, storage
  drift, and remediation diagnostics, without copying Jellyfin code or exposing
  filesystem-backed image internals.

## Redaction Boundary Verified

Admin Web data-source and route tests inject unsafe extra fields and assert they
are not projected or rendered:

- `storage_uri`
- `managed-artwork://`
- local paths and artifact roots
- raw file names
- `source_uri` / `cache_uri`
- provider URLs and query strings
- token/credential material
- content hash values

## Verification

- `npm run check --prefix apps/admin-web`
  - Passed.
- `npm run test --prefix apps/admin-web -- adminApi/client.test.ts adminApi/dataSource.test.ts App.test.tsx`
  - Passed: 185 tests.
- `cargo fmt --all -- --check`
  - Passed.
- `cargo check -p nako-api --tests`
  - Passed.
- `cargo nextest run -p nako-api admin_contract --no-fail-fast`
  - Passed: 8 tests.
- `cargo check -p nako-server --tests`
  - Passed.
- `cargo nextest run -p nako-server implemented_admin_routes_are_generated_or_explicitly_excluded --no-fail-fast`
  - Passed: 1 test.
- `cargo nextest run -p nako-server admin_managed_artwork --no-fail-fast`
  - Passed: 7 tests.
- `git diff --check`
  - Passed. Git reported Windows LF-to-CRLF working-copy warnings only.
- `python ./.trellis/scripts/task.py validate 06-08-managed-artwork-maintenance-readonly-generated-route-contract`
  - Passed.
