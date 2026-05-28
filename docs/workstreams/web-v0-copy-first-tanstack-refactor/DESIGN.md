# Web V0 Copy-First TanStack Refactor

Status: Complete
Last updated: 2026-05-28

## Why This Lane Exists

`repo-ref/nako-admin-web` has a more complete product shell than the current
thin `web/` frontend foundation. It includes media browsing, admin operations,
settings, setup, notifications, playback-adjacent pages, and future desktop or
mobile affordances. Starting from that complete generated shell is faster than
rebuilding the frontend from isolated slices.

The reference cannot become the release frontend unchanged. Its Next.js app
router, Vercel assumptions, frontend-owned TMDB route, broad mock data, raw
provider artwork URLs, single-page view-state routing, and unsupported
AI/download/music/photo/podcast controls would blur Nako's API, Addon, playback,
and Tauri boundaries.

## Product Decision

Use a copy-first refactor:

- copy `repo-ref/nako-admin-web` into `web/` as the baseline;
- keep `web/` as the browser and Tauri frontend package;
- preserve or reintroduce Nako API and Tauri shell boundaries from the previous
  `web/` foundation;
- treat Next.js as an import/bootstrap shell only;
- replace Next/server-runtime assumptions with a static TanStack-driven app;
- keep shadcn/Radix primitives where they provide accessibility and interaction
  value;
- use TanStack Router, Query, Table, Virtual, and optionally Form/Store for
  routing, server state, dense admin tables, large media grids, forms, and
  bounded cross-route UI state.

## Target State

- `web/` is a static browser/Tauri product frontend.
- Major surfaces are route-owned and code-split:
  - `/media/*`
  - `/admin/*`
  - `/setup`
  - `/account`
  - future `/desktop/*` or route groups only when backed by Tauri capability.
- Media routes consume Public Client API and `@nako/sdk`.
- Admin routes consume the generated Admin API contract.
- Shared UI components do not import Public Client or Admin DTOs.
- Tauri packages static frontend assets and does not require a Next/Node server
  sidecar.
- The app keeps v0's useful product inventory but marks unsupported flows as
  fixture/planned or removes them until backend authority exists.
- Bundle and startup cost are first-class gates, not late polish.

## In Scope

- Copy `repo-ref/nako-admin-web` into `web/`.
- Convert the copied app to a static packaging path suitable for browser and
  Tauri.
- Remove Next API routes, Vercel assumptions, and provider-secret handling.
- Replace page-state navigation with TanStack Router.
- Replace TMDB/mock hooks with Nako API boundaries and explicit fixtures.
- Preserve shadcn/Radix primitives that remain useful.
- Introduce route-level code splitting, virtualization for large grids/tables,
  and bundle budget evidence.
- Preserve Tauri profile bootstrap and native playback split from the previous
  `web/` foundation where still valid.

## Out Of Scope

- Shipping a Next server runtime inside Tauri.
- Treating WebView playback as the final desktop playback quality target.
- Implementing native desktop playback core.
- Implementing Admin UI for every v0 page in one task.
- Implementing downloader, cloud-drive transfer, AI automation, music, photos,
  podcasts, or Live TV unless a backend/API workstream accepts the capability.
- Copying Jellyfin/Plex source or assets.
- Bundling third-party provider artwork from the reference app.

## Refactor Brief

### Intent

Start from a complete v0-generated product shell to reduce frontend design
thrash, then delete runtime assumptions and mock feature claims before they
become desktop/browser product debt.

### Scope

- `web/`
- `web/src-tauri`
- `repo-ref/nako-admin-web`
- `sdk/typescript`
- generated Admin API TypeScript contracts
- this workstream's docs and evidence

### Deletion Plan

- Delete `app/api/tmdb/route.ts` and any frontend-owned provider-secret flow.
- Delete Vercel analytics/runtime assumptions.
- Delete direct provider artwork hotlinks from release routes.
- Delete or quarantine mock-only controls for destructive or broad operations.
- Delete Next.js as the long-term runtime once the copied shell is route-owned
  by TanStack Router and statically buildable.
- Delete old thin `web/src` route shell only after useful API/Tauri pieces are
  preserved or recreated.

### Boundary Plan

- Media data: Public Client API through `@nako/sdk`.
- Admin data: generated Admin API contract under an admin-only data module.
- Shared UI: no API DTO imports.
- Route state: TanStack Router search params and route params.
- Server state: TanStack Query.
- Dense tables: TanStack Table.
- Large grids/lists: TanStack Virtual.
- Tauri: local shell/profile capability only, no stored session secret, no
  bundled Next server.

### Testing Plan

- `npm --prefix web run check`
- `npm --prefix web run test`
- `npm --prefix web run build`
- `cargo test --manifest-path web/src-tauri/Cargo.toml`
- Admin contract generation/drift tests when contract output changes.
- Browser/Playwright smoke for desktop and mobile routes after visible UI
  changes.
- Bundle size review after copy, after route splitting, and at closeout.

### Risk Plan

- Feature illusion: unsupported v0 pages must be fixture/planned or removed.
- Asset provenance: release routes must not bundle or hotlink third-party
  provider artwork as Nako-owned assets.
- Tauri runtime risk: no Node/Next server sidecar in the desktop package.
- Performance risk: direct copy may be large; split routes before broad API
  integration.
- Validation risk: do not delete old `apps/admin-web` validation until its
  parity matrix is satisfied or split.

### Workflow Plan

This is a durable execution workstream. The whole lane has an active Codex
goal, but individual commits must remain bounded by TODO tasks and fresh gates.

## Architecture Direction

### Import Shape

Copy-first does not mean keep every reference file forever. The import phase may
temporarily contain Next app structure, but the target is a static app with
TanStack-owned routes.

The transition can pass through:

```text
web/
  app/                 # temporary copied Next app shell
  components/          # copied v0 components, then promoted or deleted
  lib/                 # copied utilities plus Nako API modules
  src-tauri/           # retained Tauri shell
```

The final shape should converge toward:

```text
web/
  src/
    routes/
    surfaces/
    api/
    components/
    lib/
  src-tauri/
```

### Stack Decisions

- Keep React 19.
- Prefer static Vite/TanStack Router as the final build/runtime shape.
- Keep Next only as a short-lived import shell if it helps preserve the copied
  product surface during early refactors.
- Keep shadcn/Radix controls when they solve accessibility and interaction
  details.
- Use TanStack ecosystem for routing, query/server-state, tables,
  virtualization, and route-aware state.

### Performance Rules

- No single ever-growing app chunk after route splitting.
- Large poster grids, search results, logs, and tables must be virtualized
  before they are treated as product-ready.
- Heavy AI, markdown, chart, calendar, carousel, or media dependencies stay out
  of the initial route bundle unless the route needs them.
- Browser and Tauri builds both need evidence.

## Closeout Condition

This lane can close when:

- the v0 app has been copied into `web/`;
- Next/Vercel/provider-secret assumptions are removed or quarantined;
- route ownership has moved to TanStack Router or an accepted final static
  route architecture;
- Nako API boundaries are restored for first Media/Admin slices;
- Tauri packages the static frontend without a Next server sidecar;
- unsupported v0 feature surfaces are mapped to live/fixture/planned/deferred;
- bundle evidence and browser/Tauri smoke are recorded;
- remaining work is split to narrower lanes.

Closeout status: complete as of 2026-05-28. The copied v0 shell is now the
`web/` baseline, release runtime assumptions are quarantined, top-level route
ownership lives in the TanStack shell, first Public Client and Admin API seams
are restored, Tauri builds from static assets, and the remaining copied feature
surfaces are tracked as explicit follow-ons rather than implied live product.
