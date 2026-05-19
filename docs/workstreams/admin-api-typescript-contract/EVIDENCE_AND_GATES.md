# Admin API TypeScript Contract Evidence And Gates

Status: Active
Last updated: 2026-05-19

## Smallest Current Repro

For the planning slice:

```bash
git diff --check
```

For the first implementation slice:

```bash
cargo check -p taru-api --examples
cargo nextest run -p taru-api admin --no-fail-fast
cd apps/admin-web
npm run check
npm run test
```

## Gate Set

### Contract Generator Gate

```bash
cargo fmt --all -- --check
cargo check -p taru-api --tests
cargo check -p taru-api --examples
cargo nextest run -p taru-api admin --no-fail-fast
cargo nextest run -p taru-api typescript --no-fail-fast
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
git diff --name-only -- crates/taru-client-protocol sdk/typescript
```

This proves the Public Client SDK remains separate. A changed
`sdk/typescript/src/index.ts` is allowed only when public client routes changed
for unrelated reasons; this lane must not add admin routes to that package.

### Closeout Gate

```bash
git diff --check
```

Broaden to package or workspace checks when the implementation changes shared
schema helpers or route inventory behavior outside `taru-api`.

## Evidence Anchors

- `docs/workstreams/admin-api-typescript-contract/README.md`
- `docs/workstreams/admin-api-typescript-contract/DESIGN.md`
- `docs/workstreams/admin-api-typescript-contract/TODO.md`
- `docs/workstreams/admin-web-console/ADMIN_API_MATRIX.md`
- `docs/adr/0025-openapi-public-client-sdk-contract.md`
- `docs/adr/0027-admin-api-boundary-for-web-console.md`
- `crates/taru-api/src/sdk.rs`
- `crates/taru-api/src/openapi.rs`
- `apps/admin-web/src/adminApi`
- `sdk/typescript`

## Evidence Log

- 2026-05-19: AATC-010 opened. The completed admin-web baseline is split from
  Admin API TypeScript contract generation. The default direction is an
  app-local generated contract under `apps/admin-web` backed by `taru-api`,
  with explicit separation from the Public Client TypeScript SDK.
