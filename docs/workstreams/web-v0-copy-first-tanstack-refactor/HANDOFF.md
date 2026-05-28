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

## Next Task

WVTR-020: copy `repo-ref/nako-admin-web` into `web/` while preserving reusable
Tauri and Nako API boundary assets for later reattachment.

## Key Constraints

- Do not copy Jellyfin/Plex source or assets.
- Do not ship Next API routes, Vercel assumptions, or frontend provider secrets.
- Do not treat unsupported v0 pages as live capabilities.
- Keep Nako's Addon vocabulary.
- Tauri must not require a Next/Node server sidecar at closeout.
- Commit only bounded, verified task slices.
