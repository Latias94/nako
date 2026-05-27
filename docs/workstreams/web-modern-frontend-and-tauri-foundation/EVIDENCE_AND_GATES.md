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
