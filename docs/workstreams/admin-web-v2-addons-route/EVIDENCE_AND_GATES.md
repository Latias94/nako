# Admin Web V2 Addons Route - Evidence And Gates

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

Use Playwright against `/addons` at desktop and mobile viewports. Verify
nonblank route content, status filter state, no document-level horizontal
overflow, and empty unsafe rendered-text scan.

## Evidence Anchors

- `apps/admin-web/src/App.tsx`
- `apps/admin-web/src/adminApi/dataSource.ts`
- `apps/admin-web/src/features/addons/`
- `apps/admin-web/src/App.test.tsx`
- `apps/admin-web/src/adminApi/dataSource.test.ts`

## Results

### 2026-05-25 - Frontend Contract Generation

```bash
cd apps/admin-web
npm run generate:admin-api
```

Result: passed. The generated Admin API contract remained compatible with
`AdminAddonsQuery` and the existing Addon read models.

### 2026-05-25 - TypeScript Check

```bash
cd apps/admin-web
npm run check
```

Result: passed. TypeScript accepted the `/addons` route wiring,
`AdminDataSource.loadAddons`, `AddonsRouteSummary`, and route tests.

### 2026-05-25 - Targeted Tests

```bash
cd apps/admin-web
npm run test -- App.test.tsx adminApi/dataSource.test.ts
```

Result: passed. Vitest reported 2 test files and 45 tests passing. Coverage
includes URL status query mapping, route rendering, filter updates, fallback,
data-source fallback, and unsafe rendered-text exclusions.

### 2026-05-25 - Full Unit Tests

```bash
cd apps/admin-web
npm run test
```

Result: passed. Vitest reported 4 test files and 58 tests passing.

### 2026-05-25 - Production Build

```bash
cd apps/admin-web
npm run build
```

Result: passed. Vite produced the production bundle. Vite reported the existing
chunk-size advisory for a post-minification chunk above 500 kB.

### 2026-05-25 - Repository Hygiene

```bash
git diff --check
```

Result: passed. Git reported only LF/CRLF working-copy warnings, with no
whitespace errors.

### 2026-05-25 - Browser Smoke

```bash
cd apps/admin-web
npx --no-install playwright-cli -s=admin-addons-smoke open about:blank
npx --no-install playwright-cli -s=admin-addons-smoke run-code --filename=..\\..\\target\\admin-web-v2-addons-smoke\\addons-smoke.js
```

Result: passed. Desktop and mobile checks for `/addons` found:

- title: `Nako Admin Console`;
- route heading: `Addons`;
- `Addon registry` and `Subtitle Lab` visible;
- document-level horizontal overflow: false;
- unsafe rendered-text scan: empty;
- browser console errors: 0.

Screenshots:

- `target/admin-web-v2-addons-smoke/desktop-addons.png`
- `target/admin-web-v2-addons-smoke/mobile-addons.png`

The smoke used Playwright request routing for the Addon Admin API reads so the
frontend exercised the live route/data-source path without depending on a
separate local Nako server process.

### 2026-05-25 - Review

Workstream compliance: no blocking findings. `/addons` is route-owned,
read-only, uses generated `AdminAddonsQuery` status filtering, and keeps Addon
Admin API calls behind `AdminDataSource`.

Code quality: no blocking findings. The route uses a safe `AddonsRouteSummary`
that excludes base URLs, hosted page URLs, paths, env var names, snippets,
commands, raw manifests, and raw token values. UI-level rendering derives the
install-boundary message from lifecycle booleans instead of trusting raw
backend text.

Missing gates: none for this lane's target state.

Residual risk: mutation flows are intentionally deferred. Live-backend visual
evidence with an independently running Nako server was not captured in this
lane.
