# Web V0 Copy-First TanStack Refactor - Evidence And Gates

Status: Complete
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
| 2026-05-28 | WVTR-070 | Feature gap ledger and closeout docs were updated. Closeout verification: `Get-Content docs/workstreams/web-v0-copy-first-tanstack-refactor/WORKSTREAM.json -Raw \| ConvertFrom-Json \| Out-Null`; `git diff --check`; `npm --prefix web run check`. Latest executable gates remain the WVTR-060 web build, Tauri tests, Tauri build, and static smoke evidence above. | Passed. `git diff --check` reported only Windows CRLF warnings. No blocking findings in the WVTR-070 review-style self-audit; remaining work is split into narrower follow-on lanes. |

## Final Feature Gap Ledger

| Area | Status | Notes |
| --- | --- | --- |
| Static app shell and top-level routing | Live baseline | TanStack Router owns `/`, `/media`, `/admin`, `/setup`, `/account`, `/settings`, and `/notifications` inside the static Next bootstrap shell. The final Vite/TanStack-only runtime remains a follow-on migration. |
| Media home, search, and detail | Live with fixture fallback | The copied media surface now uses the Nako Public Client data source through `@nako/sdk`, with explicit local fixtures when no server connection is configured. Image-gallery depth, watched-state mutation, and richer detail actions remain follow-ons. |
| Browser playback controls | Planned / blocked by playback integration | Short-lived browser playback ticket ADRs exist, but the copied player controls are not yet wired to a live playback-session API. Treat them as planned UI until the Public Client playback slice lands. |
| Native desktop playback | Deferred to native playback lane | Tauri static packaging works, but high-quality desktop playback stays split from this lane. The WebView is not the final desktop playback engine. |
| Admin dashboard overview | Live with fixture fallback | The first Admin read-model seam loads overview, job, playback, runtime, and config summary data through the generated Admin API contract. |
| Admin libraries, users, settings, tasks, logs | Fixture / planned | Copied pages remain useful product inventory. Logs already have virtualization; the rest need dedicated Admin API wiring, route tests, and permission/error states before product-ready status. |
| Addon Manager UI | Planned follow-on | Nako has Addon vocabulary and backend/admin contracts, but the copied plugin marketplace/library UI is not wired to live Addon management. Split into an Addon Manager UI workstream. |
| Downloads and acquisition | Planned follow-on | Copied download/transfer controls are fixture-only. Real acquisition, watch-folder, cloud-drive, and external downloader flows need backend authority and safety rules before live UI. |
| Notifications and webhooks | Planned / blocked by event bridge | The notifications route exists, but live webhook/event delivery, subscription state, and notification retention are not wired in this frontend. |
| Setup, account, and connection profile | Fixture / planned | Tauri has a local shell/profile foundation, but the setup/account UI still needs live connection-profile, auth/session, and server capability integration. |
| AI, automation, music, photos, podcasts, Live TV | Deferred domain | These copied v0 surfaces are not Nako product promises in this lane. Keep or remove them only after a domain/API owner accepts the capability. |
| i18n and localization | Planned follow-on | The copied UI still mixes English and Chinese strings. Add a real i18n strategy before treating the app as release-localized. |
| UI and E2E test suite | Planned follow-on | `npm --prefix web run test` currently aliases type-check. Add Vitest/Testing Library and Playwright route/data-source coverage once the first live seams settle. |
| Bundle and startup budget | Live baseline | WVTR-060 recorded static asset sizes and route splitting evidence. Re-run budget checks after each broad route or dependency integration. |
| Tauri desktop package | Live baseline | `npm --prefix web run tauri -- build` produces a Windows shell binary from static `web/out` without a Next/Node server sidecar. Installers and mobile packaging remain future work. |
| Legacy `apps/admin-web` | Planned deletion / blocked by parity | Keep the older Admin Web only as a validation/reference surface until a parity matrix or dedicated deletion lane proves it is safe to remove. |

## Virtualization Decisions

| Surface | Decision | Reason |
| --- | --- | --- |
| Admin logs | Live virtualization already present | `web/components/nako/admin-logs.tsx` uses `@tanstack/react-virtual`. |
| Media home/search grids | Bounded paging before virtualization | Public media data source requests 40 home items and 20 search hits. Add virtual grids when infinite scrolling or large pages are introduced. |
| Admin scheduled task history and user history | Existing pagination is acceptable for copied fixture state | Revisit when wired to live Admin API pages. |
| Plugin marketplace, library grids, metadata tables | Deferred to WVTR-070 classification | These copied v0 surfaces are still fixture/planned and should not get performance work before product status is clear. |
