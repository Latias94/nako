# Evidence

## Implementation

- Added generated Admin route key `managedArtworkIngestProcessNext` for
  `POST /admin/v1/artwork/ingests/process-next`.
- Removed the final explicit Admin route exclusion and simplified
  `admin_contract_route_exclusions()` to return an empty list.
- Emitted `ProcessManagedArtworkIngestResponse` and
  `ManagedArtworkArtifactSummary` in the generated Admin TypeScript contract
  body.
- Regenerated:
  - `apps/admin-web/src/adminApi/generated/contract.ts`
  - `web/src/api/admin/generated/contract.ts`
- Added `AdminApiClient.processNextManagedArtworkIngest()` using the generated
  route key and `POST {}`.
- Added a focused Admin Web client test covering generated route usage, empty
  body, processed response typing, empty queue response typing, and unsafe
  fixture terms.

## Jellyfin Comparison

- Jellyfin exposes elevated Scheduled Task start/stop commands through a
  controller that delegates to a task manager/worker.
- Nako's existing `process-next` route already followed a thin Admin handler to
  app-service boundary; this slice typed that command without adding UI
  controls or changing worker execution behavior.

## Verification

- `cargo fmt --all -- --check` passed.
- `git diff --check` passed.
- `python ./.trellis/scripts/task.py validate .trellis/tasks/06-08-managed-artwork-process-next-generated-route-contract`
  passed.
- `cargo check -p nako-api --tests` passed.
- `cargo nextest run -p nako-api admin_contract --no-fail-fast` passed with 8
  tests.
- `cargo check -p nako-server --tests` passed.
- `cargo nextest run -p nako-server implemented_admin_routes_are_generated_or_explicitly_excluded --no-fail-fast`
  passed with 1 test.
- `cargo nextest run -p nako-server admin_process_next_managed_artwork_ingest --no-fail-fast`
  passed with 3 tests.
- `npm run check --prefix apps/admin-web` passed.
- `npm run test --prefix apps/admin-web -- adminApi/client.test.ts` passed with
  34 tests.
- `rg` confirmed no remaining `AdminRouteExclusionSuffix` or
  `ADMIN_ROUTE_EXCLUSION_SUFFIXES` references in `admin_contract.rs`.

## Spec Updates

- Updated `.trellis/spec/nako-api/backend/quality-guidelines.md` with the
  generated Managed Artwork process-next Admin contract and the current zero
  Admin route exclusion baseline.
- Updated `.trellis/spec/admin-web/frontend/routes-forms-and-tests.md` with the
  Managed Artwork process-next client command contract.
