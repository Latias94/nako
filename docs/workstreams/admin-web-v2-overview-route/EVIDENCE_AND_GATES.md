# Admin Web V2 Overview Route - Evidence And Gates

Status: Closed
Last updated: 2026-05-25

## Smallest Current Repro

```bash
cd apps/admin-web
npm run test -- App.test.tsx adminApi/dataSource.test.ts
```

## Gate Set

### Targeted Iteration Gate

```bash
cd apps/admin-web
npm run check
npm run test -- App.test.tsx adminApi/dataSource.test.ts
```

### Frontend Package Gate

```bash
cd apps/admin-web
npm run generate:admin-api
npm run check
npm run test
npm run build
```

### Repository Hygiene Gate

```bash
git diff --check
```

### Browser Smoke Gate

Use Playwright against `/overview` and `/` at desktop and mobile viewports.
Verify nonblank route content, root redirect, no document-level horizontal
overflow, and empty unsafe rendered-text scan.

## Evidence Anchors

- `apps/admin-web/src/App.tsx`
- `apps/admin-web/src/adminApi/dataSource.ts`
- `apps/admin-web/src/features/overview/`
- `apps/admin-web/src/App.test.tsx`
- `apps/admin-web/src/adminApi/dataSource.test.ts`

## Results

### 2026-05-25 - Frontend Contract Generation

```bash
cd apps/admin-web
npm run generate:admin-api
```

Result: passed. The generated Admin API contract remained compatible with the
Overview route and existing `AdminOverviewResponse` read model.

### 2026-05-25 - TypeScript Check

```bash
cd apps/admin-web
npm run check
```

Result: passed. TypeScript accepted the default route, route-local
`loadOverview` seam, Overview page, and tests.

### 2026-05-25 - Targeted Tests

```bash
cd apps/admin-web
npm run test -- App.test.tsx adminApi/dataSource.test.ts
```

Result: passed. Vitest reported 2 test files and 35 tests passing. Coverage
includes default `/` redirect to `/overview`, Overview route fallback,
route-rendered unsafe field exclusions, and route-local data-source fallback.

### 2026-05-25 - Full Unit Tests

```bash
cd apps/admin-web
npm run test
```

Result: passed. Vitest reported 4 test files and 48 tests passing.

### 2026-05-25 - Production Build

```bash
cd apps/admin-web
npm run build
```

Result: passed. Vite produced the production bundle.

### 2026-05-25 - Repository Hygiene

```bash
git diff --check
```

Result: passed. Git reported only LF/CRLF working-copy warnings, with no
whitespace errors.

### 2026-05-25 - Browser Smoke

```bash
npx --no-install playwright-cli -s admin-overview-smoke open http://127.0.0.1:5173/overview
npx --no-install playwright-cli -s admin-overview-smoke resize 1440 1000
npx --no-install playwright-cli -s admin-overview-smoke screenshot --filename=target/admin-web-v2-overview-smoke/desktop-overview.png
npx --no-install playwright-cli -s admin-overview-smoke goto http://127.0.0.1:5173/
npx --no-install playwright-cli -s admin-overview-smoke screenshot --filename=target/admin-web-v2-overview-smoke/desktop-root.png
npx --no-install playwright-cli -s admin-overview-smoke resize 390 844
npx --no-install playwright-cli -s admin-overview-smoke screenshot --filename=target/admin-web-v2-overview-smoke/mobile-overview.png
npx --no-install playwright-cli -s admin-overview-smoke goto http://127.0.0.1:5173/
npx --no-install playwright-cli -s admin-overview-smoke screenshot --filename=target/admin-web-v2-overview-smoke/mobile-root.png
```

Result: passed. Desktop and mobile checks for `/overview` and `/` both found:

- title: `Overview`;
- root path after redirect: `/overview`;
- `Storage backends`, `Metadata providers`, and `Anime Vault` visible;
- document-level horizontal overflow: false;
- unsafe rendered text scan: empty;
- browser console errors: 0.

Screenshots:

- `target/admin-web-v2-overview-smoke/desktop-overview.png`
- `target/admin-web-v2-overview-smoke/desktop-root.png`
- `target/admin-web-v2-overview-smoke/mobile-overview.png`
- `target/admin-web-v2-overview-smoke/mobile-root.png`

Note: no live Admin API backend was attached during browser smoke, so the
visual smoke exercised the deterministic mock fallback path and the SPA
fallback/non-JSON notice path.

### 2026-05-25 - Review

Workstream compliance: no blocking findings. `/overview` is route-owned,
`/` redirects to it, API access remains behind `AdminDataSource`, and no new
backend overview fields were added.

Code quality: no blocking findings. The page renders only existing
`AdminOverviewResponse` summary fields, keeps metric state labels
domain-facing, and tests prove behavior through route/data-source seams.

Missing gates: none for this lane's target state.

Residual risk: live backend visual evidence was not captured because no Admin
API server was attached to the local frontend smoke.
