# Web V0 Copy-First TanStack Refactor - Evidence And Gates

Status: Active
Last updated: 2026-05-28

## Gate Policy

This lane intentionally starts with a large copy. Every follow-up task must
reduce ambiguity: runtime assumptions, API boundaries, route ownership,
performance, or desktop packaging.

## Baseline Gates

```bash
git status --short
npm --prefix web run check
npm --prefix web run test
npm --prefix web run build
cargo test --manifest-path web/src-tauri/Cargo.toml
```

## Contract Gates

Run when Admin or Public Client API generated artifacts change:

```bash
npm --prefix web run generate:admin-api
cargo nextest run -p nako-api admin_web_generated_contract_matches_generator_output
```

## Browser And Desktop Gates

- Browser/Playwright smoke for `/media`, `/admin`, `/setup`, and mobile view.
- Tauri static packaging smoke once the frontend build path is restored.
- Console output must not contain application errors.

## Performance Gates

- Record production bundle output after copy baseline.
- Record production bundle output after route-level splitting.
- Large poster grids, search results, logs, and admin tables need a
  virtualization decision before closeout.
- Heavy route-only dependencies must not stay in the initial route chunk unless
  there is a measured reason.

## Safety Checks

Release routes must not expose:

- bearer/session secrets;
- provider API keys;
- raw local paths;
- raw Source Locators;
- raw provider payloads;
- raw addon link URLs/passwords;
- Addon tokens or webhook secrets;
- FFmpeg argv/output paths/stderr;
- third-party provider artwork as bundled Nako-owned assets.

## Evidence Log

| Date | Task | Evidence | Result |
| --- | --- | --- | --- |
| 2026-05-28 | WVTR-010 | Workstream opened after user accepted copy-first refactor and autonomous commits. | Active. |
| 2026-05-28 | WVTR-020 | `npm --prefix web install`; `npm --prefix web run check`; `npm --prefix web run build`. Build output still reports a dynamic `/api/tmdb` route, which stays for WVTR-030 quarantine. | Passed. |
| 2026-05-28 | WVTR-030 | `rg -n "image\\.tmdb\\.org\|/api/tmdb\|TMDB_API_KEY\|@vercel/analytics\|process\\.env" web --glob '!tsconfig.tsbuildinfo' --glob '!node_modules/**' --glob '!.next/**' --glob '!out/**'`; `npm --prefix web run check`; `npm --prefix web run build`. Build route table contains only static `/`, `/_not-found`, and `/tv`. | Passed. |
| 2026-05-28 | WVTR-040 | `npm --prefix web install @tanstack/react-router`; `npm --prefix web run check`; `npm --prefix web run test`; `npm --prefix web run build`. Static build route table contains `/`, `/media`, `/admin`, `/setup`, `/account`, `/settings`, `/notifications`, and `/tv`. Production static smoke via `npx serve out -l tcp://127.0.0.1:3101 -s` plus `playwright-cli` covered `/media`, `/admin`, `/setup`, `/account`, Media -> Admin navigation, and 390x844 mobile `/media`; static console had 0 errors and 0 warnings. | Passed. `test` currently aliases type-check until a real UI/E2E test suite is introduced. |
| 2026-05-28 | WVTR-050 | `npm --prefix web install @nako/sdk@file:../sdk/typescript`; `npm --prefix web run check`; `npm --prefix web run test`; `npm --prefix sdk/typescript run check`; `npm --prefix web run build`; static `/media` and `/admin` smoke through `playwright-cli` with console 0 errors and 0 warnings. `rg` confirmed media routes do not import Admin DTOs and shared UI does not import API DTOs. | Passed. Media home/search/detail use Public Client data with fixture fallback; Admin dashboard uses Admin API data with fixture fallback. Deeper copied v0 pages remain fixture/planned for WVTR-070 classification. |
| 2026-05-28 | WVTR-060 | `npm --prefix web run check`; `npm --prefix web run test`; `npm --prefix web run build`; `cargo test --manifest-path web/src-tauri/Cargo.toml`; `cargo build --manifest-path web/src-tauri/Cargo.toml`; `npm --prefix web install -D @tauri-apps/cli@2.11.2`; `npm --prefix web run tauri -- build`. Top static assets by size: JS chunks 306.5 KB, 218.0 KB, 195.2 KB, 194.9 KB, 189.7 KB, framework 185.3 KB, CSS 179.3 KB. Tauri built `web/src-tauri/target/release/nako-web-shell.exe`. | Passed. Tauri uses `frontendDist: ../out` and `devUrl: http://127.0.0.1:3000`; bundle installers remain disabled. |

## Current Gap Ledger

| Area | Status | Notes |
| --- | --- | --- |
| UI tests | Planned | `npm --prefix web run test` is intentionally a type-check alias after WVTR-040. Replace with Vitest/Playwright coverage once routes and API seams stabilize. |
| Live Nako data | Partial | Media home/search/detail and the Admin dashboard have live/fixture seams. Deeper copied v0 pages still need live/fixture/planned/blocked/deferred classification in WVTR-070. |
| Next dev navigation noise | Non-release dev-only note | Next dev/Fast Refresh reports React hook noise on SPA route clicks, while production static smoke is clean. Use static export smoke as the release gate until Next is removed or replaced. |
| Feature inventory | Planned | Addon Manager UI, downloads/acquisition, notifications, native playback, i18n, and old Admin Web deletion still need WVTR-070 classification. |

## Virtualization Decisions

| Surface | Decision | Reason |
| --- | --- | --- |
| Admin logs | Live virtualization already present | `web/components/nako/admin-logs.tsx` uses `@tanstack/react-virtual`. |
| Media home/search grids | Bounded paging before virtualization | Public media data source requests 40 home items and 20 search hits. Add virtual grids when infinite scrolling or large pages are introduced. |
| Admin scheduled task history and user history | Existing pagination is acceptable for copied fixture state | Revisit when wired to live Admin API pages. |
| Plugin marketplace, library grids, metadata tables | Deferred to WVTR-070 classification | These copied v0 surfaces are still fixture/planned and should not get performance work before product status is clear. |
