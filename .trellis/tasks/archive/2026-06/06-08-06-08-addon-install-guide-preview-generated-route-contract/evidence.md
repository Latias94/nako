# Evidence

## Implementation

- Added generated Admin route key `addonInstallGuidePreview` for
  `POST /admin/v1/addons/install-guide-preview`.
- Removed `addons/install-guide-preview` from the explicit Admin route
  exclusion list.
- Added generated TypeScript wrapper DTOs:
  - `AdminAddonInstallGuidePreviewRequest`
  - `AdminAddonInstallGuidePreviewResponse`
- Regenerated:
  - `apps/admin-web/src/adminApi/generated/contract.ts`
  - `web/src/api/admin/generated/contract.ts`
- Added `AdminApiClient.previewAddonInstallGuide(request)` using
  `NAKO_ADMIN_ROUTES.addonInstallGuidePreview`.
- Added a focused Admin Web client test for the generated preview route, POST
  body, typed response, and response-side unsafe token/path probes.

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
  - Passed: 1 file, 28 tests.
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
- `cargo nextest list -p nako-server | rg "install_guide_preview|preview_addon_install_guide|addon_install_guide"`
  - Found the Addon install-guide preview and adjacent route tests.
- `cargo nextest run -p nako-server admin_addon_install_guide_preview --no-fail-fast`
  - Passed: 2 tests.
- `cargo nextest run -p nako-server admin_addon_install_guide --no-fail-fast`
  - Passed: 3 tests.

## Notes

- The server route implementation was already present and remained unchanged.
- This slice only makes the existing preview route visible through the generated
  Admin contract and typed Admin Web client.
- Addon register/install/lifecycle mutations and onboarding UI changes remain
  out of scope.
