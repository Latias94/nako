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
- the app router still exposes a dynamic `/api/tmdb` route, so runtime
  quarantine is still required before Tauri/static closeout.

## Next Task

WVTR-030: remove or quarantine Next server runtime assumptions, Vercel
assumptions, the TMDB API route, frontend provider secrets, and third-party
artwork hotlinks from the copied shell.

## Key Constraints

- Do not copy Jellyfin/Plex source or assets.
- Do not ship Next API routes, Vercel assumptions, or frontend provider secrets.
- Do not treat unsupported v0 pages as live capabilities.
- Keep Nako's Addon vocabulary.
- Tauri must not require a Next/Node server sidecar at closeout.
- Commit only bounded, verified task slices.
