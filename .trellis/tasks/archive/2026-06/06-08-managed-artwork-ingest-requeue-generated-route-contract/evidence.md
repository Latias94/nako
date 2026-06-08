# Evidence

## Implementation

- Added generated Admin route key `managedArtworkIngestRequeue` for
  `POST /admin/v1/artwork/ingests/{ingest_id}/requeue`.
- Removed only `artwork/ingests/{ingest_id}/requeue` from the explicit Admin
  route exclusion list.
- Left `artwork/ingests/process-next` as the only remaining Managed Artwork
  route exclusion.
- Emitted `RequeueManagedArtworkIngestResponse` and
  `ManagedArtworkIngestJobSummary` in the generated Admin TypeScript contract
  body.
- Regenerated:
  - `apps/admin-web/src/adminApi/generated/contract.ts`
  - `web/src/api/admin/generated/contract.ts`
- Added `AdminApiClient.requeueManagedArtworkIngest(ingestId)` using the
  generated route key, encoded `ingest_id`, and `POST {}`.
- Added a focused Admin Web client test covering generated route usage, path
  parameter encoding, empty body, replay-safe response booleans, and unsafe
  fixture terms.

## Jellyfin Comparison

- Jellyfin exposes elevated Scheduled Task start/stop commands through a
  controller that delegates to a task manager/worker.
- Nako generated only the explicit retry command for a selected failed ingest.
  The direct `process-next` worker execution hook remains excluded.

## Verification

- `cargo fmt --all -- --check` passed.
- `git diff --check` passed.
- `python ./.trellis/scripts/task.py validate .trellis/tasks/06-08-managed-artwork-ingest-requeue-generated-route-contract`
  passed.
- `cargo check -p nako-api --tests` passed.
- `cargo nextest run -p nako-api admin_contract --no-fail-fast` passed with 8
  tests.
- `cargo check -p nako-server --tests` passed.
- `cargo nextest run -p nako-server implemented_admin_routes_are_generated_or_explicitly_excluded --no-fail-fast`
  passed with 1 test.
- `cargo nextest run -p nako-server admin_managed_artwork_ingest_requeue --no-fail-fast`
  passed with 1 test.
- `npm run check --prefix apps/admin-web` passed.
- `npm run test --prefix apps/admin-web -- adminApi/client.test.ts` passed with
  33 tests.

## Spec Updates

- Updated `.trellis/spec/nako-api/backend/quality-guidelines.md` with the
  generated Managed Artwork ingest requeue Admin contract and the current
  single remaining Managed Artwork route exclusion.
- Updated `.trellis/spec/admin-web/frontend/routes-forms-and-tests.md` with the
  Managed Artwork ingest requeue client command contract.
