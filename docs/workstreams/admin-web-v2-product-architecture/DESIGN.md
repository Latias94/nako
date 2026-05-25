# Admin Web V2 Product Architecture

Status: Active
Last updated: 2026-05-25

## Why This Lane Exists

Admin Web V0 proved that Nako can ship a separate Vite/React/TypeScript admin
surface, consume an app-local generated Admin API contract, and fall back
section-by-section to safe mock data. That was the right tracer. It is now too
flat for V2.

The current app has one large `App.tsx`, anchor-style navigation, route labels
without real route ownership, and several dense workflows sharing one screen.
That shape makes it harder to add filters, pagination, detail pages, review
flows, and mutation safety without turning the console into an unmaintainable
dashboard.

## Relevant Authority

- `CONTEXT.md`
- `PRODUCT.md`
- `DESIGN.md`
- `docs/adr/0027-admin-api-boundary-for-web-console.md`
- `docs/workstreams/admin-web-console/`
- `docs/workstreams/admin-api-typescript-contract/`
- `docs/workstreams/nako-brand-identity/README.md`
- `apps/admin-web/README.md`

## Problem

- Product context existed only inside workstream-specific documents, so UI
  design tools lacked a root-level Nako product and design baseline.
- The Admin Web scaffold is route-shaped in copy but single-page in structure.
- Data fetching is manually composed, which is acceptable for the first
  read-model batch but weak for V2 filters, cache invalidation, retries, and
  mutations.
- Tables, forms, review plans, destructive confirmations, detail views, and
  skeleton/error states do not yet have a shared component vocabulary.
- V2 needs an explicit feature-first frontend stance so implementation does
  not spend early effort on bespoke product polish.

## Target State

When this lane closes, Admin Web V2 should have:

- root-level Nako product and design context for future UI work;
- a route-first information architecture for the admin console;
- a stack decision for routing, server-state, table, component, and styling
  layers;
- an implementation plan that slices by operator workflow, not by frontend
  layer;
- evidence gates for docs-only, frontend-only, and backend/API-changing
  slices;
- explicit follow-ons for any API gaps or desktop packaging.

## In Scope

- Product and design baselines for Admin Web.
- Current app inventory and V2 architecture research.
- Route model and navigation hierarchy for V2.
- UI component vocabulary and data-state policy.
- Stack decision candidates:
  - existing Vite + React + TypeScript foundation;
  - TanStack Query for server state;
  - TanStack Router or React Router for route ownership;
  - TanStack Table for large filterable admin tables;
  - Tailwind CSS v4 plus shadcn/ui as the first UI assembly baseline;
  - Tauri as a later packaging option, not an opening assumption.
- First implementation slice selection after the design baseline is accepted.

## Out Of Scope

- Full Admin Web rewrite in the opening slice.
- Runtime settings mutation without accepted Admin API semantics.
- Catalog repair mutations without a separate backend/API design.
- Public Client API or SDK changes.
- Addon Hosted Page embedding as trusted admin UI.
- Desktop shell packaging before the browser-hosted admin app is production
  shaped.

## Starting Assumptions

| Assumption | Confidence | Evidence | Consequence if wrong |
| --- | --- | --- | --- |
| V2 should keep `apps/admin-web` as the web workspace. | High | `admin-web-console` closeout and app scaffold. | Re-open app ownership and deployment docs. |
| Generated Admin API contract stays app-local for now. | High | `admin-api-typescript-contract` closeout. | Split package strategy before UI work. |
| Route-first structure is needed before deeper workflows. | High | Current `App.tsx` is a single large screen with multiple workflow domains. | Continuing in one file increases coupling and test cost. |
| Vite + React + TypeScript remains the base stack. | High | Existing package and validation scripts. | A replacement stack would require a separate migration lane. |
| Styling/component additions should follow product/design context, not precede it. | High | `PRODUCT.md` and `DESIGN.md` were missing before this lane. | UI would likely drift into generic dashboard patterns. |
| Desktop packaging is useful but premature. | Medium | Admin Web has not yet proven V2 route/data architecture. | If operators require desktop first, split a Tauri packaging workstream. |

## Architecture Direction

V2 should keep the current web app and generated Admin API boundary, then split
the UI by routes and workflow-owned data adapters.

V2 should be feature-first. Use shadcn/ui-style primitives and dashboard/admin
patterns to compose pages quickly, then productize UX after core admin
workflows exist. This means:

- use the official shadcn dashboard blocks as acceptable layout and component
  references;
- use `shadcn-admin` as a reference or extraction source for app shell,
  sidebar, table, form, settings, and route patterns when the license and
  copied-code ownership are recorded;
- avoid a wholesale fork unless a separate decision accepts its app structure,
  dependencies, and maintenance cost;
- keep Nako-specific work focused on domain language, generated Admin API data,
  redaction, fallback states, route ownership, and tokenized theme roles;
- defer bespoke motion, custom component art direction, and broad visual
  polishing until feature coverage is useful.

Recommended target packages inside `apps/admin-web/src`:

- `adminApi/`: generated contract, fetch client, endpoint-specific adapters,
  and redaction-preserving wire mappings.
- `routes/`: route modules that own loader/query keys, page layout, filters,
  and route-level empty/error states.
- `features/`: workflow modules such as overview, libraries, catalog
  governance, metadata profiles, jobs, playback, storage, automation, addons,
  network, and settings.
- `components/`: shared UI primitives and composed admin components with no
  Admin API ownership.
- `design/`: design tokens, semantic roles, and component-state conventions.
- `test/`: browser/test utilities and redaction fixtures.

Server state should move away from one global `Promise.all` load. V2 should
prefer route-local queries with stable query keys, section-level fallback, and
explicit retry behavior. Mutation flows must require typed request models,
clear optimistic-update rules, and redaction tests.

## First V2 Product Direction

The first route-first vertical slice should be small and operator-visible:
either Jobs or Media Libraries.

Jobs is the lower-risk first V2 candidate because it already has generated
Admin API list types, filters are natural, and mutations can stay deferred.
Media Libraries is product-important but now touches metadata profiles, scan
actions, NFO operations, and future runtime config edits, so it has a larger
API and safety surface.

## Closeout Condition

This lane can close when:

- `PRODUCT.md` and `DESIGN.md` are accepted as root design context;
- V2 research and stack recommendations are documented;
- route-first IA and first vertical proof are selected;
- the chosen proof slice lands or is split into a narrower implementation
  workstream;
- frontend gates and any touched Rust/API gates pass freshly;
- remaining work is split or explicitly deferred in `HANDOFF.md`.
