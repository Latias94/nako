# Web V0 Copy-First TanStack Refactor - Handoff

Status: Active
Last updated: 2026-05-28

## Current State

The user accepted a copy-first frontend refactor:

- `repo-ref/nako-admin-web` should be copied into `web/`;
- the copied app is the product baseline, not just a design reference;
- Next.js may be used as a short-lived bootstrap shell;
- the target is still a future-facing browser/Tauri app with TanStack route,
  query, table, and virtualization boundaries;
- performance and desktop packaging must be considered before the frontend
  grows further.

Current baseline status:

- `repo-ref/nako-admin-web` has been copied into `web/`;
- `npm --prefix web run check` and `npm --prefix web run build` both pass;
- Google Fonts and Vercel analytics assumptions were removed from the copied shell;
- the copied shell now uses local fixture media and local artwork resolution;
- the Next TMDB API route has been removed;
- `next.config.mjs` uses static export output, and the build route table has
  only static routes;
- TanStack Router now owns top-level `/`, `/media`, `/admin`, `/setup`,
  `/account`, `/settings`, and `/notifications` surfaces inside the static
  Next shell;
- production static smoke covered desktop and mobile routes with no console
  errors or warnings;
- media home/search/detail now load through a Nako Public Client data source
  with local fixture fallback instead of an inline TMDB-shaped hook;
- the Admin dashboard now loads through an Admin API read-model data source
  with local fixture fallback;
- the web package uses `@nako/sdk` from `sdk/typescript`; Next is pinned to
  webpack mode because Turbopack cannot resolve the local TS SDK package on
  Windows in this bootstrap shell;
- Tauri now points at Next static export output (`web/out`) and `npm --prefix
  web run tauri -- build` builds `nako-web-shell.exe` without a Node sidecar;
- bundle-size and virtualization decisions are recorded in `EVIDENCE_AND_GATES.md`.

## Next Task

WVTR-070: finish the feature gap ledger and close or split remaining frontend
follow-ons.

Current caveats:

- `npm --prefix web run test` is a type-check alias until real UI/E2E tests
  are introduced.
- Deeper copied v0 feature pages are still fixture/planned until WVTR-070
  classifies them explicitly.
- Next dev/Fast Refresh can report React hook noise on SPA route clicks, but
  the production static smoke is clean.
- Many copied v0 features still need live/fixture/planned/blocked/deferred
  classification in WVTR-070.

## Key Constraints

- Do not copy Jellyfin/Plex source or assets.
- Do not ship Next API routes, Vercel assumptions, or frontend provider secrets.
- Do not treat unsupported v0 pages as live capabilities.
- Keep Nako's Addon vocabulary.
- Tauri must not require a Next/Node server sidecar at closeout.
- Commit only bounded, verified task slices.
