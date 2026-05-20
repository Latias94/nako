# Taru Admin Web

This is the first real web app scaffold for Taru's administration console.
It is intentionally separate from Rust server crates and from the Public
Client SDK.

## Boundary

- Live Admin API reads go through `src/adminApi/client.ts`.
- Mock and planned data live under `src/adminApi/mockData.ts`.
- The first live slice is `GET /admin/v1/overview`.
- Build-time environment variables must not contain admin tokens or secrets.
- `VITE_TARU_ADMIN_API_BASE_URL` may point to a local Taru server during
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
`src/adminApi/generated/contract.ts` from `taru-api`. Do not edit generated
contract output by hand.
