# Web Modern Frontend And Tauri Foundation - Evidence And Gates

Status: Active
Last updated: 2026-05-28

## Gate Policy

This lane starts as a docs-first replacement plan and then becomes a frontend
execution lane. Do not claim shipped frontend behavior until implementation
tasks record fresh command and browser evidence.

## Planning Gates

- `python -m json.tool docs/workstreams/web-modern-frontend-and-tauri-foundation/WORKSTREAM.json`
- `git diff --check -- docs/workstreams/web-modern-frontend-and-tauri-foundation docs/workstreams/README.md`

## Frontend Gates

Expected once `web/` exists:

- `npm --prefix web run check`
- `npm --prefix web run test`
- `npm --prefix web run build`
- Browser/Playwright smoke for desktop and mobile route shells.
- Bundle-size review after route-level code splitting is introduced.

## Contract Gates

Run when API generators or generated contracts change:

- Public Client SDK generation command used by `sdk/typescript`.
- Admin API TypeScript contract generation command used by the frontend.
- Focused `cargo nextest run` or `cargo test` filters for changed `nako-api`
  contract tests.

Keep `apps/admin-web` validation available until `web/` has equivalent
coverage:

- `npm --prefix apps/admin-web run check`
- `npm --prefix apps/admin-web run test`
- `npm --prefix apps/admin-web run build`

## Tauri Gates

Expected once Tauri foundation exists:

- `npm --prefix web run build`
- Tauri check/build smoke chosen by the implementation task.
- Platform notes for Windows first, with Linux/macOS/mobile split only when
  those targets become active.

## Evidence Log

| Date | Task | Evidence | Result |
| --- | --- | --- | --- |
| 2026-05-27 | WMFT-010 planning open | Read existing Admin Web package shape, v0 package shape, v0 `app/page.tsx`, v0 TMDb API route, `components.json`, and related client-surface workstreams. | Active replacement lane opened. |
| 2026-05-27 | WMFT-010 validation | `python -m json.tool docs/workstreams/web-modern-frontend-and-tauri-foundation/WORKSTREAM.json`; `git diff --check -- docs/workstreams/web-modern-frontend-and-tauri-foundation docs/workstreams/README.md` | Pass. `git diff --check` emitted the existing LF/CRLF warning for `docs/workstreams/README.md` only. |
| 2026-05-27 | Current old Admin validation baseline | `npm --prefix apps/admin-web run check`; `npm --prefix apps/admin-web run test -- --runInBand`; `npm --prefix apps/admin-web run build` | Pass. Test command emitted npm warning for unknown `--runInBand`; build emitted large single-chunk warning around the current prototype bundle. |
| 2026-05-28 | WMFT-020 web scaffold | `npm install` in `web`; `npm run verify` in `web` | Pass. React 19/Vite 8/Tailwind v4/TanStack shell validates with 1 test file, 2 tests. Production output: JS 361.52 kB minified / 113.77 kB gzip; CSS 15.42 kB / 3.91 kB gzip. |
| 2026-05-28 | WMFT-020 Tauri shell | `npm run tauri -- icon ..\assets\brand\nako-app-icon-1024.png`; `cargo check --manifest-path web/src-tauri/Cargo.toml` | Pass. Tauri v2 shell compiles after isolating it from the root Rust workspace with its own `[workspace]`; icons are generated from the Nako-owned brand asset. |
| 2026-05-28 | WMFT-020 browser smoke | `playwright-cli open http://127.0.0.1:5173/admin`; `playwright-cli resize 1280 900`; `playwright-cli resize 390 844`; `playwright-cli goto http://127.0.0.1:5173/media`; screenshots `.playwright-cli/web-wmft-020-admin-fixed.png` and `.playwright-cli/web-wmft-020-media-mobile-fixed.png` | Pass. Media and Admin routes render, mobile viewport renders, the Admin light surface no longer inherits dark text/background incorrectly, and console logs contain only React DevTools info after favicon fix. |
| 2026-05-28 | WMFT-030 design shell | `npm --prefix web run verify`; `cargo check --manifest-path web/src-tauri/Cargo.toml`; `rg "rounded-xl\|Vite\|Next\|old Admin\|follow-on\|WMFT\|scaffold\|Fixture\|fixture\|Ticket seam\|Product line\|Validation retained\|Validation\|Tauri target\|prototype\|\brelease shell\b\|No frontend-only\|bg-black\|#000\|#fff\|hover:bg-white\|bg-white" web/src` | Pass. Verify produced 1 test file / 3 tests passing and production output JS 368.03 kB minified / 115.10 kB gzip, CSS 19.21 kB / 4.89 kB gzip. Tauri shell still compiles. The text/style scan returned no matches. |
| 2026-05-28 | WMFT-030 browser smoke | `playwright-cli open http://127.0.0.1:5173/media`; desktop/mobile screenshots `.playwright-cli/wmft-030-media-desktop-fixed.png`, `.playwright-cli/wmft-030-media-mobile.png`; `playwright-cli goto http://127.0.0.1:5173/admin`; screenshots `.playwright-cli/wmft-030-admin-desktop.png`, `.playwright-cli/wmft-030-admin-mobile.png`; `playwright-cli console` | Pass. Media and Admin render with active navigation, readable dark/light themes, no obvious overlap at 1280x720 and 390x844, and console output contains only React DevTools info. First media smoke exposed a CSS custom-property self-reference; `--media-*` tokens now map into the shell without cycling. |
| 2026-05-28 | WMFT-040 API boundary TDD | `npm --prefix web run test -- api-boundaries`; `cargo nextest run -p nako-api admin_web_generated_contract_matches_generator_output` | Pass. Added tests proving browser playback tickets use the Public Client SDK boundary, Admin calls use generated `/admin/v1/*` routes, section fallback stays fixture-scoped, and result payloads do not contain bearer/admin tokens. Rust generator drift check now covers both `apps/admin-web` and `web/src/api/admin/generated/contract.ts`. |
| 2026-05-28 | WMFT-040 package verification | `npm --prefix web run verify`; `cargo check --manifest-path web/src-tauri/Cargo.toml` | Pass. Verify runs `generate:admin-api`, check, 2 test files / 5 tests, and build. Production output: JS 368.03 kB minified / 115.10 kB gzip; CSS 19.21 kB / 4.89 kB gzip. Tauri shell still compiles. |
| 2026-05-28 | WMFT-050 Media Web slice | `npm --prefix web run verify`; `cargo check --manifest-path web/src-tauri/Cargo.toml` | Pass. Verify runs generated Admin contract, check, 2 test files / 6 tests, and build. Production output: JS 399.22 kB minified / 123.74 kB gzip; CSS 19.74 kB / 4.96 kB gzip. Tauri shell still compiles. Bundle growth is expected from first `@nako/sdk` route usage and should be revisited with route-level splitting. |
| 2026-05-28 | WMFT-050 Media browser smoke | `playwright-cli open http://127.0.0.1:5173/media`; screenshots `.playwright-cli/wmft-050-media-home-desktop.png`, `.playwright-cli/wmft-050-media-home-mobile.png`; route screenshots `.playwright-cli/wmft-050-media-libraries-desktop.png`, `.playwright-cli/wmft-050-media-library-detail.png`, `.playwright-cli/wmft-050-media-item-detail.png`, `.playwright-cli/wmft-050-media-watch-desktop.png`, `.playwright-cli/wmft-050-media-watch-mobile.png`; `playwright-cli console` | Pass. Media routes render fixture-backed API data, source picker, and browser ticket state with no obvious text overlap at desktop or mobile viewport. Console output contains only React DevTools info. |
| 2026-05-28 | WMFT-060 Admin product slice | `npm --prefix web run verify`; `cargo check --manifest-path web/src-tauri/Cargo.toml` | Pass. Verify runs generated Admin contract, check, 2 test files / 7 tests, and build. Production output: JS 404.33 kB minified / 124.53 kB gzip; CSS 20.15 kB / 5.01 kB gzip. Tauri shell still compiles. |
| 2026-05-28 | WMFT-060 Admin browser smoke | `playwright-cli open http://127.0.0.1:5173/admin`; screenshots `.playwright-cli/wmft-060-admin-overview-desktop.png`, `.playwright-cli/wmft-060-admin-jobs-desktop.png`, `.playwright-cli/wmft-060-admin-addons-desktop.png`, `.playwright-cli/wmft-060-admin-settings-desktop.png`, `.playwright-cli/wmft-060-admin-settings-mobile.png` | Pass. Admin routes render fixture-backed Admin API data and remain read-only; desktop and mobile screenshots show no obvious overlap. |
| 2026-05-28 | WMFT-070 runtime and Tauri tests | `npm --prefix web run verify`; `cargo test --manifest-path web/src-tauri/Cargo.toml`; `cargo check --manifest-path web/src-tauri/Cargo.toml` | Pass. Verify runs generated Admin contract, check, 3 test files / 11 tests, and build. Production output: JS 411.41 kB minified / 126.68 kB gzip; CSS 20.33 kB / 5.10 kB gzip. Tauri Rust tests validate server URL normalization, unsafe URL rejection, and invalid environment profile handling. |
| 2026-05-28 | WMFT-070 Tauri desktop smoke | `npm --prefix web run tauri -- info`; `npm --prefix web run tauri -- build` | Pass. Windows environment reports WebView2 148.0.3967.83, MSVC, Rust 1.95, Tauri 2.11.2, and matching JS/Rust package versions. Release build runs the web build and produces `web/src-tauri/target/release/nako-web-shell.exe`. |
| 2026-05-28 | WMFT-070 setup browser smoke | `playwright-cli open http://127.0.0.1:5173/setup`; screenshots `.playwright-cli/wmft-070-setup-desktop.png`, `.playwright-cli/wmft-070-setup-mobile.png`; invalid URL run-code check; `playwright-cli console` | Pass. Setup route renders the server profile form at 1280x820 and 390x844 without obvious overlap, invalid `file:///library` input shows the expected http/https validation error, and console output contains only React DevTools info. |
| 2026-05-28 | WMFT-080 old Admin retirement plan | `npm --prefix apps/admin-web run verify`; `npm --prefix web run verify`; `cargo nextest run -p nako-api admin_web_generated_contract_matches_generator_output` | Pass. Old Admin validation remains healthy: 6 Vitest files / 160 tests, check, generated contract, and build passed with the known single large-chunk warning. New `web/` verify passed with 3 files / 11 tests. Admin contract drift test passed while comparing both old and new generated contract copies. |

## Redaction And Safety Checks

The new frontend must never expose these in normal user-facing views:

- bearer tokens;
- session secrets;
- password hashes or reset tokens;
- raw local filesystem paths;
- raw Source Locators;
- provider payloads;
- Addon tokens or webhook secrets;
- FFmpeg paths, argv, output paths, or raw stderr;
- storage credentials;
- unsafe external URLs containing credentials.

Asset checks:

- bundled assets must be Nako-owned, generated for Nako, or clearly licensed
  for redistribution;
- provider artwork should be served through Nako's Managed Artwork/Public
  Client image routes, not vendored from the v0 reference;
- fixture imagery must be marked as fixture-only.
