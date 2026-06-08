# Evidence

## Implementation

- Added generated Admin route key `managedArtworkArtifactPublish` for
  `POST /admin/v1/artwork/artifacts/{artifact_id}/publish`.
- Removed `artwork/artifacts/{artifact_id}/publish` from the explicit Admin
  route exclusion list.
- Regenerated:
  - `apps/admin-web/src/adminApi/generated/contract.ts`
  - `web/src/api/admin/generated/contract.ts`
- Added `AdminApiClient.publishManagedArtworkArtifact(artifactId)` using the
  generated route key and `PublishSelectedArtworkResponse`.
- Added a focused Admin Web client test for encoded `artifact_id`, POST method,
  empty request body, typed response, and response-side unsafe storage/path
  probes.

## Commands

- `cargo run -q -p nako-api --example emit-admin-typescript-contract -- --output apps/admin-web/src/adminApi/generated/contract.ts`
  - Passed.
- `cargo run -q -p nako-api --example emit-admin-typescript-contract -- --output web/src/api/admin/generated/contract.ts`
  - Passed.
- `cargo fmt --all`
  - Passed.
- `npm run check --prefix apps/admin-web`
  - Passed.
- `npm run test --prefix apps/admin-web -- adminApi/client.test.ts`
  - Passed: 1 file, 29 tests.
- `cargo check -p nako-api --tests`
  - Passed.
- `cargo fmt --all -- --check`
  - Passed.
- `git diff --check`
  - Passed.
- `cargo nextest run -p nako-api admin_contract --no-fail-fast`
  - Passed: 8 tests.
- `cargo check -p nako-server --tests`
  - Passed.
- `cargo nextest run -p nako-server implemented_admin_routes_are_generated_or_explicitly_excluded --no-fail-fast`
  - Passed: 1 test.
- `cargo nextest run -p nako-server admin_publish_managed_artwork_artifact --no-fail-fast`
  - Passed: 1 test.

## Notes

- The server route implementation was already present and remained unchanged.
- This slice only makes the existing artifact publication route visible through
  the generated Admin contract and typed Admin Web client.
- Candidate acceptance, ingest process/requeue controls, stray-file
  remediation, artifact cleanup, and UI mutation workflows remain out of scope.
