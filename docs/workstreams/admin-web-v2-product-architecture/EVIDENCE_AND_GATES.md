# Admin Web V2 Product Architecture Evidence And Gates

Status: Active
Last updated: 2026-05-25

## Current Slice Gate

AWV2-040 component extraction:

```bash
cd apps/admin-web
npm run generate:admin-api
npm run check
npm run test
npm run build
```

This proves the generated Admin API contract is current, extracted V2
component/data code type-checks, focused Vitest coverage passes, and the Vite
production bundle builds.

Latest evidence:

- 2026-05-25: `git diff --check` passed. Git warned that
  `docs/workstreams/README.md` will be normalized from LF to CRLF the next
  time Git touches it.
- 2026-05-25: `npm run generate:admin-api` passed.
- 2026-05-25: `npm run check` passed.
- 2026-05-25: `npm run test` passed: 3 files, 24 tests.
- 2026-05-25: `npm run build` passed.
- 2026-05-25: Playwright smoke against `http://127.0.0.1:5174/` passed for
  desktop `1440x1000` and mobile `390x844`; evidence screenshots were written
  to `target/admin-web-v2-smoke/desktop.png` and
  `target/admin-web-v2-smoke/mobile.png`. Checks covered nonblank Jobs route,
  URL filter update, deterministic fallback label, no document-level
  horizontal overflow, and no visible unsafe terms such as raw tokens, local
  paths, or raw locator fields.
- 2026-05-25: AWV2-040 `npm run check` passed.
- 2026-05-25: AWV2-040 `npm run test` passed: 4 files, 26 tests.
- 2026-05-25: AWV2-040 `npm run build` passed.
- 2026-05-25: AWV2-040 Playwright smoke passed at desktop `1440x1000` and
  mobile `390x844`; evidence screenshots were written to
  `target/admin-web-v2-smoke/desktop-awv2-040.png` and
  `target/admin-web-v2-smoke/mobile-awv2-040.png`. Checks covered nonblank
  Jobs route, URL filter update, deterministic fallback notice, no
  document-level horizontal overflow, and no visible unsafe terms.

## Frontend Gate Set

Use when `apps/admin-web` changes:

```bash
cd apps/admin-web
npm run generate:admin-api
npm run check
npm run test
npm run build
```

What this proves:

- generated Admin API contract stays synchronized;
- TypeScript route/data/component code type-checks;
- Vitest coverage passes;
- Vite production build succeeds.

## Browser Gate

Use for any visible UI change:

```bash
cd apps/admin-web
npm run dev -- --host 127.0.0.1 --port 5174
```

Then verify with Browser or Playwright at:

```text
http://127.0.0.1:5174/
```

Required viewports:

- desktop: `1440x1000`
- mobile: `390x844`

Checks:

- no blank page;
- no horizontal overflow except intentional table scroll;
- no text overlap;
- focusable controls have visible focus;
- live/mock/fallback labels are truthful;
- no secrets, tokens, unsafe paths, or raw provider bodies are rendered.

## Rust/Admin API Gates

Use only when server, Admin API DTOs, or generated contract source changes:

```bash
cargo fmt --all -- --check
cargo nextest run -p nako-api admin_contract --no-fail-fast
cargo nextest run -p nako-api typescript --no-fail-fast
```

Add focused `nako-server` gates for any touched Admin API route.

## Review Gate

Run `review-workstream` before accepting implementation completion. Review
must check:

- workstream scope;
- route/data ownership;
- shadcn/ui composition stays feature-first instead of becoming custom polish;
- copied `shadcn-admin` code or patterns have license/provenance notes when
  they are used directly;
- generated contract separation from Public Client API;
- sensitive data redaction;
- missing test or browser evidence;
- UI consistency against `PRODUCT.md` and `DESIGN.md`.

## Evidence Anchors

- `PRODUCT.md`
- `DESIGN.md`
- `docs/workstreams/admin-web-v2-product-architecture/DESIGN.md`
- `docs/workstreams/admin-web-v2-product-architecture/V2_RESEARCH.md`
- `apps/admin-web/src/adminApi/generated/contract.ts`
- `apps/admin-web/src/adminApi/client.ts`
- `apps/admin-web/src/adminApi/dataSource.ts`
- `apps/admin-web/src/App.tsx`
- `apps/admin-web/src/features/jobs/JobsPage.tsx`
- `apps/admin-web/src/components/ui/`
- `apps/admin-web/src/components/layout/`
- `apps/admin-web/src/design/tokens.css`
- `apps/admin-web/src/legacy/LegacyDashboard.tsx`
