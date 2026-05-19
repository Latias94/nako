# Admin Web Console Evidence And Gates

Status: Completed
Last updated: 2026-05-19

## Planning Gates

For documentation-only planning slices:

```bash
git diff --check
```

Review checks:

- `DESIGN.md` uses Taru domain language from `CONTEXT.md`.
- `V0_CONTEXT.md` does not choose a front-end framework.
- Admin console scope is not confused with the flagship playback client.
- Route families are described as product routes, not stable API guarantees.
- Public Client API and Admin API responsibilities remain separated.

## Future API Gates

When Admin API changes are introduced, expected validation should include
focused package checks and tests for touched crates, then broader gates if
route behavior or shared DTOs change.

Likely commands:

```bash
cargo fmt --all -- --check
cargo check -p taru-api --tests
cargo check -p taru-server --tests
cargo nextest run -p taru-api --no-fail-fast
cargo nextest run -p taru-server --no-fail-fast
git diff --check
```

M52 focused API commands:

```bash
cargo fmt --all -- --check
cargo check -p taru-api --tests
cargo nextest run -p taru-api --no-fail-fast
cargo check -p taru-server --tests
cargo nextest run -p taru-server http::tests::system --no-fail-fast
cargo nextest run -p taru-api public_openapi --no-fail-fast
cargo nextest run -p taru-api typescript_sdk_excludes_admin_internal_and_secret_surfaces --no-fail-fast
git diff --check
git diff --name-only -- crates/taru-client-protocol
```

Broaden to workspace gates when API ownership, route behavior, or shared
protocol crates are touched:

```bash
cargo check --workspace --tests
cargo nextest run --workspace --no-fail-fast
```

## Future Frontend Gates

The exact commands depend on the chosen web stack. The first real web
implementation should provide:

- type-check command;
- lint command if configured;
- unit/component test command if configured;
- browser smoke verification for primary routes;
- visual checks for desktop and mobile widths;
- redaction checks for secrets/tokens/local paths in mock fixtures and UI.

AWC-060 frontend commands:

```bash
cd apps/admin-web
npm install
npm run check
npm run test
npm run build
npm run dev -- --host 127.0.0.1 --port 5174
npx --no-install playwright-cli open http://127.0.0.1:5174/
npx --no-install playwright-cli resize 1440 1000
npx --no-install playwright-cli resize 390 844
```

## Evidence Anchors

- `docs/workstreams/admin-web-console/DESIGN.md`
- `docs/workstreams/admin-web-console/ADMIN_API_MATRIX.md`
- `docs/adr/0027-admin-api-boundary-for-web-console.md`
- `docs/workstreams/admin-web-console/V0_CONTEXT.md`
- v0 prompt captured in `HANDOFF.md`
- `apps/admin-web`
- `apps/admin-web/src/adminApi/client.ts`
- `apps/admin-web/src/adminApi/dataSource.ts`
- `apps/admin-web/src/adminApi/mockData.ts`
- `apps/admin-web/README.md`
- `docs/workstreams/admin-api-typescript-contract/`

## Evidence Log

- 2026-05-17: AWC-030 completed through ADR 0027. Admin-only routes should use
  `/admin/v1/*`, admin DTOs stay in `taru-api`, future Admin API contracts are
  generated separately from Public Client OpenAPI/SDK artifacts, and
  `taru-client-protocol` remains reserved for Public Client API concepts.
- 2026-05-17: Documentation gate passed with `git diff --check`; public
  protocol boundary verified by checking `crates/taru-client-protocol` had no
  changed files.
- 2026-05-17: M52 opened as AWC-035 to implement the first read-only
  `/admin/v1/*` route through `GET /admin/v1/overview`.
- 2026-05-17: AWC-035 / M52 completed. `GET /admin/v1/overview` is wired in
  `taru-server`, uses admin-owned DTOs in `taru-api::admin`, and returns safe
  storage, metadata-provider, runtime, and startup summaries without root URI,
  secret, token, raw provider response, or output path fields. Focused
  validation passed: `cargo fmt --all -- --check`, `cargo check -p taru-api
  --tests`, `cargo nextest run -p taru-api --no-fail-fast` with 14 tests,
  `cargo check -p taru-server --tests`, `cargo nextest run -p taru-server
  http::tests::system --no-fail-fast` with 5 tests, `git diff --check`, and
  `git diff --name-only -- crates/taru-client-protocol` with no changed files.
- 2026-05-17: AWC-040/AWC-050 completed for M53. `V0_CONTEXT.md` records the
  first prototype data-source split, and `HANDOFF.md` captures a concise
  v0.dev prompt that keeps the prototype framework-neutral and admin-focused.
- 2026-05-19: AWC-060 completed. The real admin web app scaffold now lives in
  `apps/admin-web` using Vite, React, and TypeScript. The first live boundary
  is `GET /admin/v1/overview` through `src/adminApi/client.ts`; remaining first
  prototype data is centralized as mock/planned data under `src/adminApi`.
  Validation passed: `npm install`, `npm run check`, `npm run test` with 5
  tests, `npm run build`, Playwright CLI smoke at `http://127.0.0.1:5174/`,
  desktop viewport 1440x1000 with no horizontal overflow, mobile viewport
  390x844 with no horizontal overflow, and browser console with no errors after
  adding the app favicon. The app README records that build-time Vite
  environment variables must not contain admin tokens or secrets.
- 2026-05-19: AWC-070 completed. `apps/admin-web/src/adminApi/client.ts` now
  has typed methods for `GET /admin/v1/catalog/governance/items`,
  `/events`, `/jobs`, `/playback/sessions`, `/playback/runtime`,
  `/storage/staging`, and `/system/config`. `dataSource.ts` composes those
  read models with section-level live/mock fallback instead of a whole-page
  fallback. The UI surfaces Catalog Governance, Automation Events, playback
  runtime/session data, storage staging rows, and system config summaries with
  source labels and a fallback notice. Validation passed: `npm run check`,
  `npm run test` with 9 tests, `npm run build`, `git diff --check`, and
  Playwright CLI smoke at desktop 1440x1000 and mobile 390x844 with no
  horizontal overflow and no browser console errors.
- 2026-05-19: Lane closeout. AWC-010 through AWC-070 are complete. Follow-on
  Admin API TypeScript contract generation is split to
  `docs/workstreams/admin-api-typescript-contract/` so future admin web slices
  do not deepen a hand-written DTO surface.
