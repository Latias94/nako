# Admin Web Route Registry Shell Chunk Splitting

## Goal

Reduce the Admin Web initial shell bundle by moving route-local wiring out of
`App.tsx` while preserving the current TanStack Router ownership model,
URL-owned search behavior, route-level i18n catalog loading, and route tests.

## What I Already Know

- The previous Admin Web i18n catalog split removed the single large
  route-agnostic messages chunk.
- The latest observed production build still emitted an `index-*.js` chunk of
  about 488 kB.
- `App.tsx` owns the route tree and currently imports many route search types,
  normalization helpers, route wrapper functions, and feature adapter factories.
- The frontend spec requires route pages to stay lazy-loaded and route-only
  runtime code to stay out of the initial Admin shell.

## Requirements

- Keep all existing Admin Web routes and search param semantics unchanged.
- Preserve the existing `RouteI18n` namespace declarations for route pages,
  including multi-namespace routes such as library detail and playback support.
- Move route-local wrapper/runtime wiring into lazy-loaded route modules where
  doing so reduces static imports from the Admin shell.
- Keep `App.tsx` focused on shell composition, top-level route registration,
  context, and shared search normalization primitives.
- Do not introduce new router, form, validation, or i18n dependencies.
- Do not change backend APIs, generated Admin API contracts, or route URLs.

## Acceptance Criteria

- [x] `npm run check --prefix apps/admin-web` passes.
- [x] `npm run test --prefix apps/admin-web` passes.
- [x] `npm run build --prefix apps/admin-web` passes.
- [x] Existing route tests continue to cover URL normalization and route i18n
      catalog demand-loading.
- [x] Production build output shows route-local chunks for moved route modules
      and no regression to a monolithic route catalog chunk.
- [x] The final diff does not convert type-only route contracts into runtime
      imports in the Admin shell.

## Definition of Done

- Tests and build are green.
- Any durable route-splitting convention discovered during implementation is
  recorded in `.trellis/spec/admin-web/frontend/routes-forms-and-tests.md`.
- Trellis task context is curated and the task can be archived after commit.

## Technical Approach

Use the existing route-level bundle splitting pattern, but deepen it from page
components and i18n catalogs into route wrapper modules:

1. Inspect `App.tsx` static imports and route wrapper responsibilities.
2. Identify route wrappers that import feature-only adapters, page search types,
   or feature helpers into the shell.
3. Move those wrappers into colocated lazy route modules with disjoint ownership.
4. Keep shared search normalization helpers in a lightweight module consumed by
   both `App.tsx` and route modules.
5. Verify behavior with the Admin Web check/test/build commands and compare
   emitted chunk shape.

## Decision (ADR-lite)

**Context**: Admin Web is a validation console, but it now has enough routes and
localized workflow code that shell imports can quietly pull route-local logic
into the initial bundle.

**Decision**: Prefer route wrapper extraction over changing router libraries or
manual Vite chunk configuration. This keeps behavior aligned with the existing
TanStack Router architecture while giving the bundler natural dynamic import
boundaries.

**Consequences**: The route tree remains centralized, but more route components
will live in feature-owned wrapper modules. Tests must stay async-aware because
more route runtime code is lazy-loaded.

## Out of Scope

- Replacing TanStack Router.
- Changing route paths, query parameter names, or Admin API contracts.
- Reworking page layouts or visual design.
- Splitting backend crates or generated TypeScript contracts.
- Adding manual `rollupOptions.output.manualChunks` unless code-level splitting
  proves insufficient.

## Technical Notes

- Relevant spec: `.trellis/spec/admin-web/frontend/index.md`
- Relevant spec: `.trellis/spec/admin-web/frontend/routes-forms-and-tests.md`
- React performance priority here is bundle size: avoid broad static imports
  and defer route-only modules.
- Final production build emits lazy route module chunks plus independent
  `dataSource-*.js` and `mockData-*.js` chunks; `index-*.js` decreased from
  about 488.6 kB to 367.1 kB.
