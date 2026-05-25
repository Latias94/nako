# Admin Web V2 Research

Status: Draft
Last updated: 2026-05-25

## Current Local State

- `apps/admin-web` is a Vite, React, and TypeScript app.
- `apps/admin-web/package.json` currently uses React 19, Vite, TypeScript,
  Vitest, Testing Library, jsdom, and lucide-react.
- `apps/admin-web/src/adminApi/generated/contract.ts` is generated from
  `nako-api`, not hand-written.
- `apps/admin-web/src/adminApi/client.ts` keeps the fetch/auth/base URL
  boundary hand-written.
- `apps/admin-web/src/adminApi/dataSource.ts` composes many Admin API reads
  through section-level live/mock fallback.
- `apps/admin-web/src/App.tsx` renders most product domains in one large
  component and uses anchor navigation rather than route ownership.
- `apps/admin-web/src/styles.css` already uses OKLCH and responsive grids, but
  the color system leans warm/brown and the component vocabulary is still
  page-local.

## External Source Snapshot

Primary sources reviewed on 2026-05-25:

- React official blog: the current docs nav shows React `v 19.2`, and the
  official blog lists React 19.2 as the current release note stream.
  <https://react.dev/blog>
- Tailwind CSS official docs: Tailwind CSS docs show v4.3 and document Vite
  plugin installation through `tailwindcss` and `@tailwindcss/vite`.
  <https://tailwindcss.com/docs/installation/using-vite>
- shadcn/ui official docs: the Vite guide adds Tailwind CSS first, then uses
  the `shadcn` CLI to add source components into the project.
  <https://ui.shadcn.com/docs/installation/vite>
- shadcn/ui official examples: the dashboard example is an acceptable
  component/layout reference for Admin Web V2's first feature-first shell.
  <https://ui.shadcn.com/examples/dashboard>
- `shadcn-admin`: a Vite + TypeScript + shadcn/ui admin dashboard reference
  under the MIT license. Its README says it is not meant as a starter project,
  so use it as a reference or selective extraction source rather than assuming
  a clean wholesale fork.
  <https://github.com/satnaing/shadcn-admin>
- TanStack official docs: Query targets server-state fetching/caching,
  Router gives typed search params and route context, and Table is headless
  table/datagrid logic rather than a styled grid component.
  <https://tanstack.com/query/latest/docs/framework/react/overview>
  <https://tanstack.com/router/latest/docs/overview>
  <https://tanstack.com/table/latest/docs/introduction>
- Tauri official docs: Tauri 2 supports desktop and mobile shells using web
  frontends and native webviews, but adds packaging/security concerns that
  should follow, not lead, the Admin Web V2 architecture.
  <https://v2.tauri.app/start/>

## Stack Recommendation

Keep:

- Vite + React + TypeScript as the base. This avoids a migration before the
  product architecture is settled.
- Generated Admin API TypeScript contract as app-local source of truth for
  Admin API route constants and wire DTOs.
- Hand-written `AdminApiClient` for auth, base URL, fetch behavior, and
  redaction-safe runtime policy.
- lucide-react for icons.

Add first:

- TanStack Query for route-local server state, retry behavior, loading/error
  states, and mutation invalidation once the first V2 route is implemented.
- A real router before adding detail pages. TanStack Router is a strong fit if
  typed route params and typed search/filter state are valued; React Router is
  acceptable if team familiarity matters more than route typing.
- Tailwind CSS v4 plus shadcn/ui as the first UI composition baseline. V2
  should build admin features with standard shadcn primitives, dashboard
  blocks, tables, filters, forms, sheets, and dialogs before investing in
  bespoke Nako-specific component polish.

Add selectively:

- TanStack Table for admin tables that need sorting, filtering, pagination,
  column visibility, and row selection. Avoid it for short static lists.
- `shadcn-admin` patterns for sidebar, route layout, settings pages, and table
  workflows. Do not import its whole product shape blindly; extract only the
  parts that reduce feature delivery cost and record copied-code provenance.

Defer:

- Tauri packaging. It can reuse the web app later, but V2 should first prove
  route ownership, auth/token handling, redaction, and production web behavior
  in the browser.
- A packaged Admin API npm SDK. The existing app-local contract is enough until
  a second admin client exists.

## V2 Information Architecture

Primary navigation should remain familiar, but route ownership should replace
the current anchor layout:

```text
/overview
/libraries
/libraries/:libraryId
/libraries/:libraryId/metadata-profile
/catalog/governance
/catalog/governance/:itemId
/metadata/providers
/metadata/maintenance
/jobs
/jobs/:jobId
/playback/sessions
/playback/sessions/:sessionId
/playback/runtime
/storage/staging
/automation/events
/automation/generated-artifacts
/addons
/addons/new
/addons/:addonId
/addons/:addonId/tokens
/addons/:addonId/grants
/network
/settings
```

The nav can group routes into:

- Operations: Overview, Jobs, Events.
- Libraries: Media Libraries, Metadata Profile, Storage.
- Governance: Catalog Governance, Metadata, Generated Artifacts.
- Runtime: Playback, Transcode, Network.
- Extensions: Addons, Automation.
- System: Settings.

## First Implementation Candidate

Preferred first proof: `/jobs`.

Why:

- Existing generated Admin API route and DTOs exist.
- List filters map cleanly to URL search params.
- It needs a real table but no mutation semantics.
- It can prove route module, TanStack Query, filter controls, loading
  skeleton, empty state, error state, shadcn-style table composition, and
  redaction-safe row rendering.

Acceptance shape:

- `/jobs` route owns `status`, `kind`, `resource_class`, `library_id`,
  `source_id`, `limit`, and `offset` search params.
- Page uses generated `AdminJobsQuery` and `AdminJobListResponse`.
- Page is assembled from shadcn/ui-style table, filter, badge, button, and
  empty/error components with minimal Nako theming.
- Mock fallback remains deterministic and section-local.
- UI distinguishes live data, fallback data, and unavailable route state.
- Tests cover query mapping, redaction, empty state, and fallback state.

## API Gaps To Track

- Job detail by Admin API route, not only known legacy route.
- Job cancel/retry semantics only after durable runtime policy supports them.
- Catalog governance detail and repair routes.
- Media Library runtime create/edit/delete semantics.
- Metadata profile UI update workflow and safety copy.
- Playback session detail and support evidence deep link.
- Settings mutation policy.
- Addon Manager install/update/remove execution beyond planning and install
  guide output.
