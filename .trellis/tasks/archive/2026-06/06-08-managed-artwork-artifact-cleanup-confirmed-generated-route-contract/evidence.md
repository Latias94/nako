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
- `cargo nextest run -p nako-server admin_managed_artwork_cleanup --no-fail-fast`

## Notes

- `POST /admin/v1/artwork/artifacts/cleanup` now requires `confirm=true`.
- The generated route contract now includes `managedArtworkArtifactCleanup`.
- `artwork/artifacts/cleanup` was removed from explicit Admin route exclusions.
- The Admin Web client sends confirmation and pagination through query parameters and posts an empty JSON body.
- The server test proves unconfirmed cleanup is rejected before deleting artifact rows, while confirmed cleanup still removes only unselected cleanup candidates and preserves selected artwork.
