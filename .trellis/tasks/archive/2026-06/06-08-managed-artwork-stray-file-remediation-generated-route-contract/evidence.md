# Evidence

## Validation

- `npm run generate:admin-api --prefix apps/admin-web`
- `cargo run -q -p nako-api --example emit-admin-typescript-contract -- --output web/src/api/admin/generated/contract.ts`
- `cargo fmt --all`
- `cargo fmt --all -- --check`
- `git diff --check`
- `npm run check --prefix apps/admin-web`
- `npm run test --prefix apps/admin-web -- adminApi/client.test.ts`
- `cargo check -p nako-api --tests`
- `cargo nextest run -p nako-api admin_contract --no-fail-fast`
- `cargo check -p nako-server --tests`
- `cargo nextest run -p nako-server implemented_admin_routes_are_generated_or_explicitly_excluded --no-fail-fast`
- `cargo nextest run -p nako-server admin_managed_artwork_remediation --no-fail-fast`
- `python ./.trellis/scripts/task.py validate .trellis/tasks/06-08-managed-artwork-stray-file-remediation-generated-route-contract`

## Notes

- The generated route contract now includes `managedArtworkArtifactRemediateStrayFiles`.
- `artwork/artifacts/remediate-stray-files` was removed from explicit Admin route exclusions.
- The Admin Web client sends confirmation and `file_scan_limit` through query parameters and posts an empty JSON body.
- Existing server remediation tests prove `confirm=true` is required and only parseable untracked artifact files are deleted after active DB state is rechecked.
