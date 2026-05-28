# Web V0 Copy-First TanStack Refactor - Handoff

Status: Complete
Last updated: 2026-05-28

## Closed State

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
- bundle-size and virtualization decisions are recorded in `EVIDENCE_AND_GATES.md`;
- the copied v0 feature inventory is classified as live, fixture, planned,
  blocked-by-backend, or deferred-domain rather than implied product scope.

## Closeout Result

WVTR-070 is complete. This lane established the copied v0 frontend as the new
`web/` baseline, removed release-blocking runtime assumptions, restored first
Nako API boundaries, proved static browser/Tauri packaging, and split remaining
work into narrower product lanes.

Follow-on lanes:

- Migrate from the static Next bootstrap shell to a Vite/TanStack-only runtime
  after route seams settle.
- Add Vitest/Testing Library and Playwright coverage for routes, data sources,
  connection states, and critical admin/media flows.
- Wire expanded Admin pages for libraries, users, settings, tasks, logs, jobs,
  permissions, and error states.
- Build the Addon Manager UI against Nako Addon APIs instead of the copied
  plugin fixture screens.
- Build acquisition/download/watch-folder UI only after backend authority,
  safety boundaries, and credential handling are accepted.
- Wire notifications to a live event/webhook bridge and persistence model.
- Wire Public Client playback, image/detail depth, watched state, and browser
  playback-ticket flows.
- Wire setup/account to Tauri profile/bootstrap and browser auth/session
  connection state.
- Decide whether AI, automation, music, photos, podcasts, and Live TV are Nako
  domains before keeping their copied UI.
- Add a real i18n/localization strategy.
- Delete or merge the older `apps/admin-web` only after parity and validation
  evidence exists.

Residual risks:

- `npm --prefix web run test` is still a type-check alias.
- The production static build is clean, but Next dev/Fast Refresh can still
  report React hook noise on SPA route clicks until the bootstrap shell is
  removed.
- Many copied pages are intentionally fixture or planned status; do not present
  them as live backend-backed product surfaces.

## Key Constraints

- Do not copy Jellyfin/Plex source or assets.
- Do not ship Next API routes, Vercel assumptions, or frontend provider secrets.
- Do not treat unsupported v0 pages as live capabilities.
- Keep Nako's Addon vocabulary.
- Tauri must not require a Next/Node server sidecar at closeout.
- Commit only bounded, verified task slices.
