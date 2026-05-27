# Web Modern Frontend And Tauri Foundation

Status: Active
Last updated: 2026-05-28

## Why This Lane Exists

Nako now has enough backend and client-contract surface to stop treating the
web UI as a temporary Admin verification console. The v0-generated
`repo-ref/nako-admin-web` captures the desired direction better than the old
prototype: media-first browsing, a richer playback-oriented shell, an operator
surface, settings, notifications, and future desktop/mobile affordances.

The product decision is now explicit:

- `web/` becomes the new product frontend for browser release and Tauri shell
  packaging.
- `apps/admin-web` stays only as a validation/prototype surface until the new
  frontend reaches agreed parity and contract gates.
- The v0 app is design and interaction reference material; Nako should re-author
  the implementation into its own route-first, API-driven frontend instead of
  shipping Next.js/Vercel/mock-only assumptions.

## Relevant Authority

- `CONTEXT.md`
- `PRODUCT.md`
- `DESIGN.md`
- `docs/adr/0024-inbound-token-authentication-boundary.md`
- `docs/adr/0026-native-client-shells-with-shared-rust-client-core.md`
- `docs/adr/0027-admin-api-boundary-for-web-console.md`
- `docs/adr/0032-shared-rust-client-core-transport-boundary.md`
- `docs/adr/0036-browser-playback-ticket-transport.md`
- `docs/adr/0037-local-credentials-and-opaque-sessions.md`
- `docs/workstreams/admin-web-v2-product-architecture/`
- `docs/workstreams/media-web-client-foundation/`
- `docs/workstreams/browser-playback-auth-transport/`
- `docs/workstreams/client-surface-and-access-product-architecture/`
- `docs/workstreams/web-modern-frontend-and-tauri-foundation/RUST_CAPABILITY_GAPS.md`
- `apps/admin-web`
- `repo-ref/nako-admin-web`

## Problem

The old Admin Web has proved many useful contracts, but it is no longer the
right product shell:

- It mixes Admin and Media concerns inside one Vite app built for rapid
  validation, not product-level release.
- Route ownership, generated Admin contract consumption, Public Client SDK
  usage, fixtures, layout, and CSS are tightly coupled enough that deep product
  redesign will fight the prototype shape.
- Its UI tone is operational and dense, while the release frontend needs a
  media-first experience for viewers and an admin surface for operators.
- The current build already produces a large single frontend chunk, which
  signals that route/module boundaries need to be reconsidered before the UI
  grows.

The v0 reference has the opposite problem:

- The visual and interaction direction is closer to the desired product.
- The architecture is not acceptable as-is: Next.js app routes, Vercel
  analytics, server-only TMDb route code, mock data, broad AI/download/music/TV
  surfaces, and third-party poster/backdrop assets must not be blindly shipped.
- It uses a single page state machine for major surfaces instead of durable
  routes that can be tested, deep-linked, permission-gated, and packaged.

## Target State

When this lane closes:

- `web/` is the release-oriented Nako frontend package.
- `web/` is a Vite React application using React 19, TypeScript, Tailwind v4,
  shadcn-style primitives, lucide icons, TanStack Router, TanStack Query, and
  TanStack Table where dense admin tables require it.
- Major surfaces are route-owned, not page-state-owned:
  - `/media/*` for Media Web.
  - `/admin/*` for Admin Web.
  - setup, account, settings, notifications, and future desktop routes are
    explicit route families, not hidden modal state.
- Media Web consumes Public Client API and `@nako/sdk`.
- Admin Web consumes Admin API DTOs only inside `/admin/*` modules.
- Shared UI primitives are source-agnostic; they do not import Admin API DTOs or
  Public Client DTOs.
- Tauri has a clear packaging path from the new frontend, with desktop playback
  split into WebView convenience first and native playback core later.
- Rust/server capability gaps are recorded so the frontend does not ship
  controls for unowned or unverified Jellyfin-class breadth.
- `apps/admin-web` remains available for validation while the new frontend is
  built, then is retired or archived only after parity gates are documented and
  passed.

## In Scope

- Create a new `web/` package for the product frontend.
- Choose the product frontend stack and package scripts.
- Re-author the useful v0 visual system and interaction patterns into Nako-owned
  route-first code.
- Create API boundaries for Public Client SDK, Admin API contract, fixture
  data, and live server configuration.
- Bring over proven browser playback ticket usage from the old frontend once
  the media route shell exists.
- Add a Tauri foundation path that can package the new frontend without making
  WebView playback the final desktop playback architecture.
- Track the parity and retirement gates for `apps/admin-web`.

## Out Of Scope

- Replacing the backend Public Client API or Admin API contracts in this lane.
- Copying Jellyfin, Plex, or other reference source/assets.
- Shipping the v0 Next.js application as-is.
- Shipping Vercel analytics, Next API routes, or frontend-owned TMDb secret
  handling.
- Shipping third-party movie posters, backdrops, screenshots, or provider
  artwork as bundled Nako assets.
- Implementing Addon Manager lifecycle, online media acquisition, AI
  automation, music, photos, podcasts, or TV clients unless a narrower follow-on
  accepts the backend/product boundary.
- Solving native desktop playback. This lane can prepare the Tauri shell and
  split a native-player spike, but ADR 0026 still points serious desktop
  playback toward native playback ownership.
- Tauri mobile delivery. The frontend should stay mobile-aware, but mobile
  packaging is a future lane.

## Starting Assumptions

| Assumption | Confidence | Evidence | Consequence if wrong |
| --- | --- | --- | --- |
| The new `web/` frontend is the product release line. | High | User direction on 2026-05-27. | Reopen this lane and treat `apps/admin-web` as the product app again, which would make most replacement tasks invalid. |
| The old `apps/admin-web` should stay during migration. | High | It already passes check/test/build and proves generated Admin/Public client contracts. | Removing it too early would weaken backend contract validation while `web/` is still incomplete. |
| Vite SPA is a better fit than Next.js for this frontend. | High | Nako does not need SSR for the self-hosted client, and Tauri packaging benefits from a static SPA shape. | If SSR becomes required, split an ADR before adopting server-rendered frontend runtime. |
| The v0 app is valuable mainly as UX reference. | High | It has strong screens but contains Next/Vercel/TMDb/mock/asset assumptions. | Blind import would ship unsupported features, secret-handling mistakes, and asset provenance risk. |
| Tauri should package Media Web first, not Admin Web first. | Medium | ADR 0026 and Product docs center desktop playback around media consumption. | If operator desktop packaging becomes urgent, split an Admin Console packaging lane. |

## Refactor Brief

### Intent

Replace prototype-driven frontend growth with a product frontend boundary that
can ship as browser Web, desktop Tauri shell, and future mobile-aware UI without
dragging old Admin validation code or v0 mock architecture into release code.

### Scope

- New package: `web/`.
- Future Tauri package location: `web/src-tauri` unless an ADR accepts a
  separate `apps/desktop` package.
- Reference inputs: `apps/admin-web` for proven API behavior and
  `repo-ref/nako-admin-web` for UX direction.
- Shared client dependencies: `sdk/typescript` and generated Admin API
  TypeScript contract.

### Deletion Plan

No runtime deletion happens at lane open. The planned deletions are gated:

- Remove Next.js/Vercel/TMDb route assumptions from any migrated v0 code.
- Do not ship third-party media artwork copied from the v0 reference.
- Remove mock-only product surfaces unless they are explicitly converted to
  supported API-backed or fixture-backed routes.
- Retire or archive `apps/admin-web` only after `web/` passes release frontend
  parity gates and old validation coverage has an equivalent replacement.

### Boundary Plan

- `/media/*` owns viewer media browsing, playback, personal state, and source
  selection through Public Client API.
- `/admin/*` owns operator management, jobs, settings, Addons, storage,
  network, metadata governance, and diagnostics through Admin API.
- Shared UI components are dumb primitives or composition components without
  backend DTO imports.
- Data access modules are per-surface: `mediaApi`, `adminApi`, and
  `appSession`.
- Tauri owns shell capabilities and later native playback integration; the web
  app owns route state and API queries.

### Testing Plan

- For docs: `python -m json.tool` on `WORKSTREAM.json` and `git diff --check`.
- For `web/`: package check/test/build, route tests, data-source tests, and
  Browser/Playwright visual smoke.
- For API contract changes: regenerate Public Client SDK/Admin API contract and
  run focused Rust contract tests.
- For Tauri: at minimum desktop build/check smoke once the Tauri package exists.

### Risk Plan

- Asset provenance risk: ship only Nako-owned, generated, or server-served
  Managed Artwork assets.
- Contract drift risk: keep `apps/admin-web` validation until `web/` owns
  equivalent contract checks.
- Feature illusion risk: mock-rich v0 surfaces must be tagged unsupported or
  omitted until backend/API authority exists.
- Performance risk: route-level code splitting and bundle budget are first
  class gates, not late polish.
- Playback risk: WebView playback is not accepted as the final desktop quality
  target.

### Workflow Plan

This is a durable workstream. Do not set a Codex goal for the whole lane. A
goal is appropriate for the first bounded implementation task, `WMFT-020`, once
execution starts.

## Architecture Direction

### Package Shape

The target package is:

```text
web/
  package.json
  index.html
  src/
    app/
    routes/
      media/
      admin/
      setup/
      account/
    surfaces/
      media/
      admin/
    api/
      media/
      admin/
    components/
      ui/
      shell/
    styles/
  src-tauri/        # introduced when Tauri foundation starts
```

This package should not depend on Next.js, server components, or framework API
routes. Server interaction goes through the Nako backend API.

### Route Model

Use route state for product surfaces:

- `/media` home.
- `/media/libraries`.
- `/media/libraries/:libraryId`.
- `/media/items/:itemId`.
- `/media/watch/:itemId`.
- `/admin`.
- `/admin/libraries`.
- `/admin/items/:itemId`.
- `/admin/jobs`.
- `/admin/addons`.
- `/admin/settings`.

Surface switching should navigate to routes. It should not mutate a single
top-level `surface` state value that hides the current product context from the
URL.

### Data Model Boundary

Media modules may import Public Client SDK types. Admin modules may import
generated Admin contract types. Shared UI modules import neither.

Fixture data remains useful, but fixtures must be named as fixtures and must
not imply unsupported features are available on a real server.

### v0 Migration Policy

Use `repo-ref/nako-admin-web` for:

- visual direction;
- navigation vocabulary;
- interaction inventory;
- component decomposition ideas;
- route backlog and feature-gap discovery.

Do not blindly migrate:

- Next.js app route structure;
- Vercel analytics;
- `app/api/tmdb/route.ts`;
- frontend-owned provider secrets;
- mock-only feature claims;
- bundled third-party posters/backdrops/screenshots;
- unsupported Addon, AI, download, TV, music, photo, or notification behavior.

### Old Admin Web Policy

`apps/admin-web` remains useful while the new product frontend is incomplete:

- validation console for Admin API and Public Client API behavior;
- reference for existing data-source mapping and tests;
- smoke target for backend route work that already uses it.

It should not receive broad new product investment unless the work is needed to
preserve validation during migration. Once `web/` owns equivalent contract and
surface gates, open a retirement task for `apps/admin-web`.

## Closeout Condition

This lane can close when:

- `web/` exists and is the accepted release frontend package;
- browser Media Web and Admin Web have first real API-backed route families;
- Tauri foundation is either implemented or split with accepted evidence;
- old Admin Web validation coverage has an equivalent home in `web/` or CI;
- unsupported v0 features and assets are either removed, converted, or split to
  explicit follow-ons;
- evidence gates pass and remaining work is split into narrower lanes.
