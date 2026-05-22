# Admin Web Addon Onboarding Evidence and Gates

Status: Active
Last updated: 2026-05-22

## Required Gates

Rust/API gates:

- `cargo fmt --all -- --check`
- `cargo nextest run -p nako-api admin_contract --no-fail-fast`
- `cargo check -p nako-api -p nako-server --tests`

Admin Web gates from `apps/admin-web`:

- `npm run check`
- `npm test`
- `npm run build`

Repository gate:

- `git diff --check`

## Evidence Log

- 2026-05-22: Workstream opened. No implementation evidence yet.
- 2026-05-22: AWAON-020 client/data-source slice implemented. Evidence:
  `npm test -- --run src/adminApi/client.test.ts`,
  `npm test -- --run src/adminApi/dataSource.test.ts`, `npm run check`.
- 2026-05-22: AWAON-030 UI slice implemented. Evidence:
  `npm test -- --run src/App.test.tsx`.
- 2026-05-22: AWAON-040 docs slice implemented in `docs/api/HTTP_API.md` and
  `docs/guides/ADDON_AUTHOR_GUIDE.md`.
- 2026-05-22: Full closeout gates passed:
  - `cargo fmt --all -- --check`;
  - `cargo nextest run -p nako-api admin_contract --no-fail-fast`;
  - `cargo nextest run -p nako-server register_addon_routes_disabled_by_default_and_validate_contract --no-fail-fast`;
  - `cargo check -p nako-api -p nako-server --tests`;
  - `npm run check`, `npm test`, and `npm run build` in `apps/admin-web`;
  - `git diff --check`.

## Safety Evidence Required Before Close

- Addon onboarding UI defaults registration to disabled. Proven by
  `src/adminApi/client.test.ts` and `src/adminApi/dataSource.test.ts`.
- Addon onboarding does not fetch arbitrary manifest URLs. Proven by code
  inspection: onboarding accepts pasted JSON only.
- Addon onboarding does not start, stop, install, update, remove, or supervise
  sidecars. Proven by UI/data-source code paths using only `POST
  /admin/v1/addons`.
- UI and tests do not expose raw tokens, admin bearer tokens, resolved secrets,
  Source Locators, storage URIs, or local filesystem paths.
