# Web Modern Frontend And Tauri Foundation - Milestones

Status: Active
Last updated: 2026-05-28

## M0 - Direction Accepted

Status: Complete

Exit criteria:

- `web/` is documented as the product frontend release line.
- `apps/admin-web` is documented as validation/prototype only during migration.
- The v0 reference is documented as UX direction, not a shippable Next.js app.

## M1 - Product App Scaffold

Status: Complete

Exit criteria:

- `web/` exists as a Vite React package.
- The package has check/build scripts and strict TypeScript configuration.
- The package does not include Next.js, Vercel analytics, frontend TMDb API
  routes, or bundled third-party media artwork.
- The first route scaffold is route-owned rather than a single top-level
  surface state machine.
- `web/src-tauri` exists as a Tauri shell skeleton targeting the new frontend.

## M2 - Design System And Shell

Status: Complete

Exit criteria:

- Media and Admin shells are implemented with shared Nako UI primitives.
- Visual direction reflects the v0 product target while staying Nako-owned.
- Desktop and mobile viewport smoke evidence shows no broken layout, overlapped
  text, or missing primary navigation.
- Unsupported feature surfaces are absent or clearly fixture-only.

## M3 - API Boundaries

Status: Complete

Exit criteria:

- Public Client SDK consumption is isolated to Media modules.
- Generated Admin API consumption is isolated to Admin modules.
- Shared UI primitives have no backend DTO imports.
- Fixture/live data-source seams are tested.
- Contract regeneration remains repeatable.

## M4 - Media Web Product Slice

Status: Complete

Exit criteria:

- Libraries, library detail, item detail, source/version picker, browser player,
  and Continue Watching basics work against fixtures and live API seams.
- Browser playback ticket URLs are used without exposing bearer tokens, raw
  Source Locators, local paths, or permanent privileged stream URLs.
- Browser smoke evidence covers browse, detail, and player routes.

## M5 - Admin Product Slice

Status: Complete

Exit criteria:

- The new `/admin/*` surface has the first useful API-backed operator routes.
- Admin routes remain redaction-safe and own confirmation for broad/destructive
  operations.
- Admin-to-media links use stable IDs and current-principal access rules.

## M6 - Tauri Foundation

Status: Complete

Exit criteria:

- Tauri package structure is present or explicitly split to a narrower lane.
- Desktop shell build/check smoke is recorded when implemented.
- Native playback core integration is not claimed without a separate spike and
  evidence.

## M7 - Old Admin Web Retirement

Exit criteria:

- A parity matrix lists which old `apps/admin-web` validation responsibilities
  moved to `web/` or CI.
- The old app is kept, archived, or deleted by an explicit task after parity,
  not as incidental cleanup.
- Any removal stages only files owned by that task.

## M8 - Closeout

Exit criteria:

- Workstream evidence is fresh.
- Remaining product breadth is split to narrower lanes.
- `WORKSTREAM.json` status and `HANDOFF.md` accurately describe continuation.
