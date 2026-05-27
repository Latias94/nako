# Web Modern Frontend And Tauri Foundation - Handoff

Status: Closed
Last updated: 2026-05-28

## Current State

The foundation lane is closed. It reflects the accepted product direction:

- the new `web/` frontend is the release product line;
- the existing `apps/admin-web` remains only as validation/prototype support;
- `repo-ref/nako-admin-web` is valuable UX reference material but should be
  re-authored into a Nako-owned Vite SPA rather than shipped as the Next.js app;
- Tauri is part of the frontend direction, but serious desktop playback still
  requires a later native playback-core spike.

WMFT-020 through WMFT-090 are complete:

- `web/` exists as the product frontend package.
- The stack is Vite 8, React 19, Tailwind v4, shadcn-style local primitives,
  TanStack Router/Query, and lucide icons.
- Media and Admin are separate route families.
- `web/src-tauri` exists and targets the new frontend.
- Tauri icons were generated from `assets/brand/nako-app-icon-1024.png`.
- `RUST_CAPABILITY_GAPS.md` records server/Rust gaps the frontend must not fake.
- `web/src` now has Nako-owned media/admin visual tokens, active side
  navigation, a route surface switcher, product-facing disconnected states, and
  generated artwork placeholders rather than v0 assets.
- Browser smoke covered Media/Admin desktop and mobile viewports. A media CSS
  token self-reference found during smoke was fixed by splitting media tokens
  into `--media-*` and mapping them into `--app-*` inside the shell.
- `web/src/api` now owns the release frontend API boundary: Media uses
  `@nako/sdk`, Admin uses a generated `web/src/api/admin/generated/contract.ts`,
  and both surfaces expose live/fixture section results without leaking tokens.
- `nako-api` now checks that the Admin API TypeScript contract generated for
  `web/` matches the generator output, alongside the old Admin validation copy.
- Media Web routes now consume `web/src/api/media` through TanStack Query:
  Continue Watching, Libraries, Library Detail, Item Detail, source selection,
  and browser playback ticket state are driven by fixture/live results.
- Browser playback only assigns a ticket URL to `<video src>` for live results.
  Fixture tickets stay visible as state, not as a network load.
- Admin routes now consume `web/src/api/admin` through TanStack Query: Overview,
  Libraries, Jobs, Addons, and Settings render fixture/live results without
  adding unsafe mutation controls.
- Tauri now has no-secret bootstrap commands for desktop server profiles:
  `desktop_bootstrap`, `save_server_profile`, and `clear_server_profile`.
- The web runtime verifies the Public Client `/health` endpoint before storing
  a server profile, then routes Media/Admin API calls through the configured
  runtime base URL.
- Desktop bootstrap reports native playback as unavailable with
  `native_playback_core_not_integrated`, preserving the ADR 0026 split between
  WebView convenience playback and future native desktop playback.
- Windows Tauri smoke builds `web/src-tauri/target/release/nako-web-shell.exe`.
- `ADMIN_WEB_RETIREMENT_PLAN.md` records the old `apps/admin-web` parity matrix.
  The current decision is to retain it as validation-only; future deletion is
  allowed only after matrix rows are ready, moved to CI, or split to accepted
  follow-ons.
- `apps/admin-web/README.md` now labels the old app as a historical validation
  console rather than the product frontend.

## Next Recommended Work

Open narrower follow-on lanes from `CLOSEOUT.md`.

Recommended implementation choices:

- Do not mark the whole frontend migration as complete; close only this
  foundation lane.
- Split follow-ons for Addon Manager UI, Admin mutation parity, redaction
  corpus migration, Media playback state, route-level code splitting, i18n,
  and native desktop playback.
- Keep `apps/admin-web` retained according to `ADMIN_WEB_RETIREMENT_PLAN.md`.

## Key Constraints

- Media Web consumes Public Client API only.
- Admin Web consumes Admin API only.
- Shared UI primitives must not import backend DTOs.
- The old Admin Web can remain as contract validation while `web/` is built.
- The v0 "plugin" wording should be translated to Nako's `Addon` vocabulary.
- Do not claim support for AI, downloads, music, photos, TV, Addon lifecycle,
  notifications, or online discovery until those flows have backend/API
  authority or are clearly fixture-only.
- Tauri WebView playback is a convenience tier, not the final desktop playback
  quality target.

## Follow-On Candidates

- Management Context Links route matrix.
- Native desktop playback core spike.
- Credential/session UX in the new frontend.
- Addon Manager UI productization.
- Notifications center backed by real Addon/Webhook event surfaces.
- Acquisition/downloads UI after product boundaries are accepted.
- Mobile packaging and responsive polish.
- Admin mutation parity and redaction corpus migration.
- Route-level code splitting and bundle budget.
- `web/` i18n decision and implementation.
- Old Admin Web deletion only after `ADMIN_WEB_RETIREMENT_PLAN.md` is satisfied.

## Resume Notes

Before coding, re-read:

- `DESIGN.md`
- `PRODUCT.md`
- ADR 0026, ADR 0027, ADR 0032, ADR 0036, ADR 0037
- `apps/admin-web/src/surfaces/media/mediaDataSource.ts`
- `apps/admin-web/src/surfaces/media/MediaPages.tsx`
- `web/src/api/media/client.ts`
- `web/src/api/admin/client.ts`
- `web/src/api/runtime.ts`
- `web/src/api/desktop.ts`
- `web/src-tauri/src/lib.rs`
- `web/src/router.tsx`
- `docs/workstreams/web-modern-frontend-and-tauri-foundation/ADMIN_WEB_RETIREMENT_PLAN.md`
- `repo-ref/nako-admin-web/app/page.tsx`
- `repo-ref/nako-admin-web/components/nako/media-surface.tsx`
- `repo-ref/nako-admin-web/components/nako/admin-surface.tsx`
