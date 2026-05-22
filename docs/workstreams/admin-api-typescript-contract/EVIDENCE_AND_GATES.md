# Admin API TypeScript Contract Evidence And Gates

Status: Completed
Last updated: 2026-05-20

## Smallest Current Repro

For the planning slice:

```bash
git diff --check
```

For the first implementation slice:

```bash
cargo check -p nako-api --examples
cargo nextest run -p nako-api admin --no-fail-fast
cd apps/admin-web
npm run check
npm run test
```

## Gate Set

### Contract Generator Gate

```bash
cargo fmt --all -- --check
cargo check -p nako-api --tests
cargo check -p nako-api --examples
cargo nextest run -p nako-api admin --no-fail-fast
cargo nextest run -p nako-api typescript --no-fail-fast
```

This proves the generator compiles, covered admin route tests pass, and public
TypeScript SDK guard tests still reject admin/internal leakage.

### Admin-Web Consumption Gate

```bash
cd apps/admin-web
npm run check
npm run test
npm run build
```

This proves the generated contract is consumable by the web app and does not
break existing live/mock or redaction behavior.

### Separation Gate

```bash
npm run generate --prefix sdk/typescript
npm run check --prefix sdk/typescript
git diff --name-only -- crates/nako-client-protocol sdk/typescript
```

This proves the Public Client SDK remains separate. A changed
`sdk/typescript/src/index.ts` is allowed only when public client routes changed
for unrelated reasons; this lane must not add admin routes to that package.

### Closeout Gate

```bash
git diff --check
```

Broaden to package or workspace checks when the implementation changes shared
schema helpers or route inventory behavior outside `nako-api`.

## Evidence Anchors

- `docs/workstreams/admin-api-typescript-contract/README.md`
- `docs/workstreams/admin-api-typescript-contract/DESIGN.md`
- `docs/workstreams/admin-api-typescript-contract/ADMIN_CONTRACT_INVENTORY.md`
- `docs/workstreams/admin-api-typescript-contract/TODO.md`
- `docs/workstreams/admin-web-console/ADMIN_API_MATRIX.md`
- `docs/adr/0025-openapi-public-client-sdk-contract.md`
- `docs/adr/0027-admin-api-boundary-for-web-console.md`
- `crates/nako-api/src/sdk.rs`
- `crates/nako-api/src/openapi.rs`
- `apps/admin-web/src/adminApi`
- `sdk/typescript`

## Evidence Log

- 2026-05-19: AATC-010 opened. The completed admin-web baseline is split from
  Admin API TypeScript contract generation. The default direction is an
  app-local generated contract under `apps/admin-web` backed by `nako-api`,
  with explicit separation from the Public Client TypeScript SDK.
- 2026-05-19: AATC-020 completed. `ADMIN_CONTRACT_INVENTORY.md` records the
  current admin-web hand-written wire DTOs, UI-only local types, covered
  `/admin/v1/*` route/query inventory, and accepted first artifact shape:
  route constants plus wire/query interfaces without a generated fetch client.
  Validation passed: `npm run check` in `apps/admin-web` and `git diff --check`
  with CRLF warnings only.
- 2026-05-19: AATC-030 completed. `crates/nako-api/src/admin_contract.rs`
  generates route constants, query interfaces, and wire interfaces for the
  eight AWC-070 Admin API read-model routes; the emit example writes the
  app-local artifact at
  `apps/admin-web/src/adminApi/generated/contract.ts`; `client.ts` imports the
  generated response types and route constants while keeping the fetch runtime
  hand-written. Validation passed:
  `$env:CARGO_TARGET_DIR='G:\nako-cargo-target'; cargo check -p nako-api --examples`,
  `$env:CARGO_TARGET_DIR='G:\nako-cargo-target'; cargo nextest run -p nako-api admin_contract --no-fail-fast -j 2`,
  `cd apps/admin-web && npm run check`, and
  `cd apps/admin-web && npm run test`.
  Additional gates passed:
  `$env:CARGO_TARGET_DIR='G:\nako-cargo-target'; cargo nextest run -p nako-api typescript --no-fail-fast -j 2`
  and `cd apps/admin-web && npm run build`. Formatting and diff hygiene passed
  with `cargo fmt --all -- --check` and `git diff --check`.
  `cargo fmt --all` also mechanically formatted one existing import wrapping
  difference in `crates/nako-server/src/http/tests/mod.rs` so the broad
  formatting gate can remain green.
- 2026-05-20: AATC-040 completed. `apps/admin-web/src/adminApi/types.ts`
  no longer owns long-lived hand-written wire DTO definitions; it re-exports
  covered response/page DTOs from `generated/contract` while keeping
  admin-web-only view models local. `dataSource.ts` and `mockData.ts` import
  wire DTOs directly from the generated contract, and `dataSource.test.ts`
  uses generated route constants for fixture routing. Validation passed:
  `cd apps/admin-web && npm run check`, `cd apps/admin-web && npm run test`,
  `cd apps/admin-web && npm run build`, and `git diff --check`.
- 2026-05-20: AATC-050 completed. `nako-api` now has an explicit guard proving
  every generated Admin API read-model route stays out of
  `nako-client-protocol` public route inventory. `DESIGN.md`,
  `docs/api/HTTP_API.md`, and `apps/admin-web/README.md` document the
  app-local Admin API generation command and the Public Client SDK separation
  checks. Validation passed:
  `$env:TMP='F:\Temp'; $env:TEMP='F:\Temp'; cargo check -p nako-api --examples`,
  `$env:TMP='F:\Temp'; $env:TEMP='F:\Temp'; cargo nextest run -p nako-api admin_contract --no-fail-fast -j 2`,
  `$env:TMP='F:\Temp'; $env:TEMP='F:\Temp'; cargo nextest run -p nako-api typescript --no-fail-fast -j 2`,
  `cd apps/admin-web && npm run check`,
  `cd apps/admin-web && npm run test`,
  `cd apps/admin-web && npm run build`,
  `npm run generate --prefix sdk/typescript`,
  `npm run check --prefix sdk/typescript`,
  `git diff --name-only -- crates/nako-client-protocol sdk/typescript`
  with no output,
  `cargo fmt --all -- --check`, and `git diff --check`.
