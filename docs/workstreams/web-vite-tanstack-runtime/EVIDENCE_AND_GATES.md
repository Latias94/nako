# Web Vite TanStack Runtime - Evidence And Gates

Status: Complete
Last updated: 2026-05-28

## Gate Policy

This lane removes a frontend runtime dependency. Each task must prove that the
new Vite runtime preserves shipped static behavior before deleting more of the
old Next surface.

## Baseline Gates

```bash
git status --short
npm --prefix web run check
npm --prefix web run build
```

## Runtime Deletion Gate

```bash
rg -n "from ['\"]next|next/|Metadata|Viewport|next dev|next build|next start|\\.next|next-env" web --glob '!node_modules/**' --glob '!out/**' --glob '!dist/**' --glob '!tsconfig.tsbuildinfo'
```

The command should find no release source dependency on Next after WVRT-030.
Package-lock entries may appear before dependency removal and should be gone
after lockfile refresh.

## Browser And Desktop Gates

- Static browser smoke for `/media`, `/admin`, `/setup`, `/account`, `/tv`, and
  mobile `/media`.
- Console output must not contain application errors.
- Tauri must build from static Vite output without a Next/Node sidecar.

## Closeout Gates

```bash
npm --prefix web run check
npm --prefix web run build
cargo test --manifest-path web/src-tauri/Cargo.toml
npm --prefix web run tauri -- build
git diff --check
```

## Evidence Log

| Date | Task | Evidence | Result |
| --- | --- | --- | --- |
| 2026-05-28 | WVRT-010 | Workstream opened from the completed copy-first WVTR lane to remove the temporary Next bootstrap runtime. | Active. |
| 2026-05-28 | WVRT-020 | `npm --prefix web install -D vite @vitejs/plugin-react`; `npm --prefix web run check`; `npm --prefix web run build`. Vite entry files added at `web/index.html`, `web/src/main.tsx`, and `web/src/app-root.tsx`; global CSS imported from `web/src/styles/globals.css`; `/tv` added to TanStack Router. | Passed. Vite builds `web/dist` without a Next server runtime. Next dependency and app wrappers still remain for WVRT-030 deletion. |
| 2026-05-28 | WVRT-030 | `npm --prefix web uninstall next next-themes`; removed `web/app`, `web/next.config.mjs`, and `web/next-env.d.ts`; replaced `next-themes` with a local theme provider; `rg -n "\"next\"\|next-themes\|node_modules/next\|next/dist" web/package-lock.json web/package.json`; release-source Next reference scan; `npm --prefix web run check`; `npm --prefix web run build`. | Passed. Package and lockfile have no Next dependency, release source has no Next imports, `/tv` is TanStack-owned, and Vite build remains green. |
| 2026-05-28 | WVRT-040 | `npm --prefix web run check`; `npm --prefix web run build`; `cargo test --manifest-path web/src-tauri/Cargo.toml`; static smoke served `web/dist` through `npx serve dist -l tcp://127.0.0.1:3102 -s` and covered `/media`, `/admin`, `/setup`, `/account`, `/tv`, and 390x844 mobile `/media`; `npm --prefix web run tauri -- build`. | Passed. Static smoke reported content on every target route and zero console errors or warnings. Tauri uses `frontendDist: ../dist` and built `web/src-tauri/target/release/nako-web-shell.exe` without a Node sidecar. |
| 2026-05-28 | WVRT-050 | Closeout docs updated; `Get-Content docs/workstreams/web-vite-tanstack-runtime/WORKSTREAM.json -Raw \| ConvertFrom-Json \| Out-Null`; `git diff --check`; `npm --prefix web run check`; latest broader executable gates are WVRT-040 build/static smoke/Tauri evidence above. | Passed. `git diff --check` reported only Windows CRLF warnings. No blocking findings in the closeout self-review; remaining work is outside the runtime migration boundary. |

## Bundle Notes

WVRT-020 Vite build output highlights:

- `dist/index.html`: 1.06 KB, gzip 0.63 KB.
- `dist/assets/index-CABLC-Bk.css`: 193.30 KB, gzip 28.25 KB.
- `dist/assets/index-BuDxzkE0.js`: 446.91 KB, gzip 139.42 KB.
- `dist/assets/media-surface-CHPoxruI.js`: 325.52 KB, gzip 73.80 KB.
- `dist/assets/admin-surface-CZDAs5pO.js`: 196.86 KB, gzip 42.50 KB.

Final Vite build output highlights from WVRT-040:

- `dist/index.html`: 1.06 KB, gzip 0.63 KB.
- `dist/assets/index-CpBtXGNh.css`: 201.18 KB, gzip 29.27 KB.
- `dist/assets/index-saVVfjE3.js`: 447.58 KB, gzip 139.68 KB.
- `dist/assets/media-surface-BUu-Lb6j.js`: 325.52 KB, gzip 73.80 KB.
- `dist/assets/admin-surface-WSZVhXQa.js`: 196.86 KB, gzip 42.50 KB.

Runtime follow-on note: the initial `index` chunk is still large enough to
justify a later bundle optimization lane, but that is no longer tied to Next
runtime cleanup.
