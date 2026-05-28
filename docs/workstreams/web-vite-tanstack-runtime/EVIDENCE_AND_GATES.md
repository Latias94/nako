# Web Vite TanStack Runtime - Evidence And Gates

Status: Active
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

## Bundle Notes

WVRT-020 Vite build output highlights:

- `dist/index.html`: 1.06 KB, gzip 0.63 KB.
- `dist/assets/index-CABLC-Bk.css`: 193.30 KB, gzip 28.25 KB.
- `dist/assets/index-BuDxzkE0.js`: 446.91 KB, gzip 139.42 KB.
- `dist/assets/media-surface-CHPoxruI.js`: 325.52 KB, gzip 73.80 KB.
- `dist/assets/admin-surface-CZDAs5pO.js`: 196.86 KB, gzip 42.50 KB.

Record the final closeout build output again after Next deletion and Tauri
static packaging updates.
