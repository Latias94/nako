# Admin Web V2 Automation Generated Artifacts Route Evidence And Gates

Status: Closed
Last updated: 2026-05-25

## Smallest Current Repro

```bash
cd apps/admin-web
npm run test -- App.test.tsx adminApi/dataSource.test.ts
```

## Gate Set

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

Run the dev server and smoke `/automation/generated-artifacts` at:

- desktop: `1440x1000`
- mobile: `390x844`

Checks:

- route content is nonblank;
- no document-level horizontal overflow;
- source/fallback status is truthful;
- pagination state is reflected in the URL;
- unsafe rendered text scan is empty for prompt bodies, payload bodies, raw
  provider responses, source URIs, local paths, credentials, and tokens.

## Evidence Anchors

- `apps/admin-web/src/App.tsx`
- `apps/admin-web/src/features/automation/GeneratedArtifactsPage.tsx`
- `apps/admin-web/src/adminApi/client.ts`
- `apps/admin-web/src/adminApi/dataSource.ts`
- `apps/admin-web/src/App.test.tsx`
- `apps/admin-web/src/adminApi/dataSource.test.ts`

## Results

### 2026-05-25 - Targeted Route Gate

```bash
cd apps/admin-web
npm run check
npm run test -- App.test.tsx adminApi/dataSource.test.ts
```

Result: passed. TypeScript accepted the route, route-local data-source seam,
query normalization, and tests. Vitest reported 2 files and 57 tests passing.

### 2026-05-25 - Frontend Contract Generation

```bash
cd apps/admin-web
npm run generate:admin-api
```

Result: passed. The generated Admin API contract remained compatible with the
Generated Artifacts route and existing generated query/response types.

### 2026-05-25 - Full Unit Tests

```bash
cd apps/admin-web
npm run test
```

Result: passed. Vitest reported 4 files and 70 tests passing.

### 2026-05-25 - Production Build

```bash
cd apps/admin-web
npm run build
```

Result: passed. Vite produced the production bundle. Vite reported the existing
large chunk warning.

### 2026-05-25 - Repository Hygiene

```bash
git diff --check
```

Result: passed. Git reported LF/CRLF working-copy warnings only, with no
whitespace errors.

### 2026-05-25 - Browser Smoke

```bash
cd apps/admin-web
npm run dev -- --host 127.0.0.1 --port 5180
npx --no-install playwright-cli -s admin-aw-v2-remaining-routes open http://127.0.0.1:5180/automation/generated-artifacts
npx --no-install playwright-cli -s admin-aw-v2-remaining-routes resize 1440 1000
npx --no-install playwright-cli -s admin-aw-v2-remaining-routes screenshot --filename=../../target/admin-web-v2-generated-artifacts-smoke/desktop.png
npx --no-install playwright-cli -s admin-aw-v2-remaining-routes resize 390 844
npx --no-install playwright-cli -s admin-aw-v2-remaining-routes screenshot --filename=../../target/admin-web-v2-generated-artifacts-smoke/mobile.png
```

Result: passed. Desktop and mobile checks found:

- title: `Generated Artifacts`;
- `artifact-metadata-cleanup` visible;
- document-level horizontal overflow: false;
- unsafe rendered text scan: empty for prompt text, payload bodies, raw provider
  responses, raw tokens, source URIs, Windows paths, and `/Users/` paths;
- browser console errors: 0.

Screenshots:

- `target/admin-web-v2-generated-artifacts-smoke/desktop.png`
- `target/admin-web-v2-generated-artifacts-smoke/mobile.png`

Note: the Browser plugin was read first, but its Node REPL control tool was not
exposed in this session, so the smoke used the local Playwright CLI fallback.
