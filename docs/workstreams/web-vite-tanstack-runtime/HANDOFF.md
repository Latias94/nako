# Web Vite TanStack Runtime - Handoff

Status: Complete
Last updated: 2026-05-28

## Closed State

The previous copy-first lane is closed, and this lane has removed the temporary
Next bootstrap shell. `web/` now builds as a Vite static React app, TanStack
Router owns the route inventory, and Tauri consumes `web/dist`.

## Closeout Result

- Next app wrappers, Next config, Next type file, `next`, and `next-themes`
  were deleted.
- Vite owns `dev`, `build`, and preview runtime scripts.
- `/`, `/media`, `/admin`, `/notifications`, `/settings`, `/setup`,
  `/account`, and `/tv` are TanStack-owned routes.
- Static smoke passed for desktop routes and mobile `/media` with zero console
  errors or warnings.
- Tauri tests and `npm --prefix web run tauri -- build` passed against
  `frontendDist: ../dist`.

## Decisions Since Last Update

- Keep `components/`, `lib/`, and `src/api/` in place during this lane.
- WVRT-020 added `/tv` to TanStack Router so the Vite app preserves the static
  route inventory from Next.
- WVRT-030 removed Next app wrappers, Next config/type files, and `next` /
  `next-themes` dependencies.
- WVRT-040 moved Tauri from `../out` to `../dist`, passed static route smoke,
  Tauri tests, and Tauri build.
- WVRT-050 closed the runtime lane. Remaining work is not Next cleanup.

## Residual Risks

- `npm --prefix web run test` still aliases type-check.
- The first Vite app chunk is still about 448 KB before gzip. Route splitting
  exists, but a later bundle budget lane should move more shared copied code out
  of the initial chunk.
- `components/`, `lib/`, and `src/api/` were intentionally not reorganized in
  this lane to avoid mixing runtime migration with broad source-tree reshaping.

## Next Recommended Action

- Open a narrower frontend lane for either route/data-source tests or initial
  bundle optimization before wiring more copied fixture pages to live APIs.
