# Nako Web

Copy-first product frontend for Nako.

The current shell is a Vite-built static React 19 app with Tailwind 4,
shadcn-style components, TanStack Router, TanStack Query, and the Tauri desktop
shell. The release path stays static so Tauri does not require a Node sidecar.

## Commands

```bash
npm run dev
npm run check
npm run test
npm run build
npm run tauri -- build
```

`npm run test` is currently a type-check gate. Replace it with real UI/E2E
coverage when the route and API seams stabilize.

## Runtime Data

- Public media data uses the Nako Public Client SDK with local fixture fallback.
- Admin dashboard data uses the Nako Admin API contract with local fixture
  fallback.
- Deeper copied v0 surfaces are still fixture/planned until the feature gap
  ledger classifies them.
