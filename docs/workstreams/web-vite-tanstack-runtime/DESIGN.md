# Web Vite TanStack Runtime

Status: Active
Last updated: 2026-05-28

## Why This Lane Exists

The copy-first frontend refactor intentionally used Next.js as a temporary
static bootstrap shell so the complete v0-generated product surface could be
preserved quickly. That shell is now accidental complexity: route ownership is
already in TanStack Router, the app has no server routes, Tauri consumes static
assets, and Nako API boundaries no longer depend on Next.

Keeping Next would continue to add build cost, dev-server behavior differences,
Windows/Turbopack friction with the local SDK package, and app/page wrapper
files that do not express Nako's runtime architecture.

## Relevant Authority

- `docs/workstreams/web-v0-copy-first-tanstack-refactor/`
- `docs/adr/0026-native-client-shells-with-shared-rust-client-core.md`
- `docs/adr/0027-admin-api-boundary-for-web-console.md`
- `docs/adr/0032-shared-rust-client-core-app-supplied-transport.md`
- `docs/adr/0036-short-lived-browser-playback-tickets.md`
- `docs/adr/0037-local-credential-and-session-auth.md`
- `web/src-tauri/tauri.conf.json`

## Problem

`web/` still has a Next app directory, `next.config.mjs`, `next-env.d.ts`, Next
scripts, Next type plugins, and a `next` dependency even though runtime behavior
is a static TanStack app. That blurs ownership and makes the desktop shell carry
framework assumptions it no longer needs.

## Target State

- `web/` builds with Vite and React 19.
- TanStack Router owns all top-level routes currently shipped by the static app:
  `/`, `/media`, `/admin`, `/notifications`, `/settings`, `/setup`,
  `/account`, and `/tv`.
- The app has a single browser entry under `web/src/`.
- Global CSS, metadata, query provider, and ResizeObserver suppression move out
  of Next layout files into Vite-owned HTML/React entry points.
- Next app wrappers, `next.config.mjs`, `next-env.d.ts`, Next scripts, and the
  `next` dependency are removed.
- Tauri consumes `web/dist` without a Node/Next sidecar.
- Static browser smoke and Tauri build/test evidence are recorded.

## In Scope

- Add `index.html`, `vite.config.ts`, and Vite React entry files.
- Move or import `app/globals.css` from a Vite-owned location.
- Update scripts and TypeScript config for Vite.
- Delete Next-only app/page wrappers and config files.
- Preserve Nako Public Client/Admin API seams and current fixture fallbacks.
- Preserve route-level lazy loading and static packaging.
- Update docs and evidence.

## Out Of Scope

- Rewriting copied product pages or visual design.
- Wiring additional Admin/Addons/playback/download features.
- Native playback core.
- Mobile Tauri packaging.
- Replacing Tailwind, shadcn, Radix, TanStack Query, or TanStack Router.

## Starting Assumptions

| Assumption | Confidence | Evidence | Consequence if wrong |
| --- | --- | --- | --- |
| The app no longer needs Next server or app-router features. | High | Previous WVTR closeout removed API routes and uses TanStack Router for surfaces. | Reintroduce only the specific missing capability, not the whole framework. |
| Vite can resolve `@nako/sdk` from the local TypeScript package. | Medium | `@nako/sdk` now exposes a TS import path and Next webpack mode can build it. | Add a targeted package export/build adjustment or split SDK compilation. |
| Tauri can consume Vite `dist` the same way it consumed Next `out`. | High | Tauri already points at static assets and does not need a server sidecar. | Adjust `frontendDist` and dev URL only; do not bundle Node. |
| `/tv` is still a route inventory item even though it is fixture/planned. | High | Next static build still emitted `/tv`. | Add it to TanStack Router as a planned surface so migration does not silently drop it. |

## Architecture Direction

Vite becomes the only web build/runtime tool. TanStack Router remains the route
authority, with React lazy imports preserving route-level chunks. The app entry
should be small and explicit:

```text
web/
  index.html
  vite.config.ts
  src/
    main.tsx
    app-root.tsx
    ...
```

The existing `components/`, `lib/`, and `src/api/` modules can stay in place
during this lane to avoid mixing runtime migration with broad source-tree
reshaping. A later lane can converge the directory layout once the runtime is
stable.

## Closeout Condition

This lane can close when:

- Vite dev/build scripts replace Next scripts;
- no release source imports from `next` or uses `next/*`;
- TanStack Router owns every route previously emitted by the static Next build;
- `npm --prefix web run check`, `npm --prefix web run build`, browser smoke,
  Tauri tests, and Tauri build evidence pass or have documented blockers;
- bundle evidence is recorded for the Vite build;
- old Next files and dependencies are deleted.
