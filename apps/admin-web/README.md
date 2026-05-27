# Nako Admin Web

This is the historical Admin Web validation console for Nako. It is retained
to validate Admin API contracts, redaction behavior, route/query mapping, and
selected mutation flows while the product frontend moves to `web/`.

It is intentionally separate from Rust server crates and from the Public Client
SDK. It is no longer the release product frontend.

The current retirement decision lives in
`docs/workstreams/web-modern-frontend-and-tauri-foundation/ADMIN_WEB_RETIREMENT_PLAN.md`:
keep this app until equivalent validation coverage exists in `web/` or lower
level CI gates.

## Boundary

- Live Admin API reads go through `src/adminApi/client.ts`.
- Mock and planned data live under `src/adminApi/mockData.ts`.
- The live surface is validation-oriented; do not add broad product UX here
  unless it is required to preserve backend contract coverage during migration.
- Build-time environment variables must not contain admin tokens or secrets.
- `VITE_NAKO_ADMIN_API_BASE_URL` may point to a local Nako server during
  development.

Authentication is a follow-on Admin API/UI concern. Do not bake bearer tokens
into Vite environment variables.

## Commands

```bash
npm install
npm run generate:admin-api
npm run check
npm run test
npm run build
npm run dev -- --host 127.0.0.1 --port 5174
```

`npm run generate:admin-api` refreshes
`src/adminApi/generated/contract.ts` from `nako-api`. Do not edit generated
contract output by hand.

The generated Admin API contract is intentionally app-local. It is not the
Public Client TypeScript SDK in `sdk/typescript`, and Admin `/admin/v1/*`
routes must not be added to `nako-client-protocol` public route inventory.
