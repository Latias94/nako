# Admin Web V2 Playback Sessions Route - Evidence And Gates

Status: Closed
Last updated: 2026-05-25

## Smallest Current Repro

```bash
cd apps/admin-web
npm run test -- App.test.tsx adminApi/client.test.ts adminApi/dataSource.test.ts
```

## Gate Set

### Targeted Iteration Gate

```bash
cd apps/admin-web
npm run check
npm run test -- App.test.tsx adminApi/client.test.ts adminApi/dataSource.test.ts
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

Use Playwright against `/playback/sessions` at desktop and mobile viewports.
Verify nonblank route content, session rows, no document-level horizontal
overflow, and empty unsafe rendered-text scan.

## Evidence Anchors

- `apps/admin-web/src/App.tsx`
- `apps/admin-web/src/adminApi/client.ts`
- `apps/admin-web/src/adminApi/dataSource.ts`
- `apps/admin-web/src/features/playback/`
- `apps/admin-web/src/App.test.tsx`
- `apps/admin-web/src/adminApi/client.test.ts`
- `apps/admin-web/src/adminApi/dataSource.test.ts`

## Results

### 2026-05-25 - Frontend Contract Generation

```bash
cd apps/admin-web
npm run generate:admin-api
```

Result: passed. The generated Admin API contract remained compatible with the
Playback Sessions route and query DTO.

### 2026-05-25 - TypeScript Check

```bash
cd apps/admin-web
npm run check
```

Result: passed. TypeScript accepted the route, query adapters, data-source seam,
and tests.

### 2026-05-25 - Targeted Tests

```bash
cd apps/admin-web
npm run test -- App.test.tsx adminApi/client.test.ts adminApi/dataSource.test.ts
```

Result: passed. Vitest reported 3 test files and 38 tests passing.

### 2026-05-25 - Full Unit Tests

```bash
cd apps/admin-web
npm run test
```

Result: passed. Vitest reported 4 test files and 40 tests passing. Coverage
includes route search params, API client query construction, data-source
fallback, and redaction-sensitive rendering.

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
playwright-cli open http://127.0.0.1:5173/playback/sessions
playwright-cli resize 1440 1000
playwright-cli screenshot --filename=target/admin-web-v2-playback-sessions-smoke/desktop.png
playwright-cli resize 390 844
playwright-cli screenshot --filename=target/admin-web-v2-playback-sessions-smoke/mobile.png
```

Result: passed. Desktop and mobile checks both found:

- title: `Playback Sessions`;
- `session-hls` visible;
- `hls_transcode` visible;
- document-level horizontal overflow: false;
- unsafe rendered text scan: empty.

Screenshots:

- `target/admin-web-v2-playback-sessions-smoke/desktop.png`
- `target/admin-web-v2-playback-sessions-smoke/mobile.png`

Note: no live Admin API backend was available during the browser smoke, so the
visual smoke exercised the deterministic mock fallback path and the
non-JSON-response notice from Vite's SPA fallback.

### 2026-05-25 - Review

Workstream compliance: no blocking findings. The route stays read-only, owns
URL search state, and defers detail/support-evidence workflows.

Code quality: no blocking findings. Admin API access remains behind
`AdminDataSource`, and `AdminApiClient` now accepts the generated
`AdminPlaybackSessionsQuery`.

Missing gates: none for this lane's target state.

Residual risk: live backend visual evidence was not captured because no Admin
API server was attached to the local frontend smoke.
