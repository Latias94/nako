# Admin Web Main Chunk Route Splitting

## Problem

`apps/admin-web` already split the Incident Bundle page into its own route
chunk, but the main Vite chunk remains about 1.05 MB. The Admin Web surface is a
validation-oriented operator app with many route-owned pages, so cold loading
every page from `App.tsx` makes unrelated diagnostics, media, and workflow code
part of the initial bundle.

## Goal

Reduce the Admin Web main JavaScript chunk by moving non-critical route page
modules behind route-level `React.lazy` boundaries while preserving existing
TanStack Router search ownership, mock fallback behavior, i18n text, and tests.

## Scope

- Split additional route page modules imported by `apps/admin-web/src/App.tsx`
  when they are not required to construct the shell, route tree, or search
  validators.
- Keep the existing route components as the place that reads route context,
  params, search, and navigation helpers.
- Keep all page props and URL-owned search normalization behavior unchanged.
- Add or preserve route tests for lazy-loaded pages that have existing App
  coverage.
- Verify that `npm run build --prefix apps/admin-web` produces smaller route
  chunks and a reduced main chunk.

## Out Of Scope

- No design system rewrite.
- No new router framework, bundler plugin, or package dependency.
- No Admin API contract, server route, or generated client changes.
- No product UX expansion beyond bundle behavior.

## Acceptance Criteria

- TypeScript check passes for `apps/admin-web`.
- Existing Admin Web route tests pass.
- Admin Web build passes and emits multiple route chunks beyond
  `IncidentBundlePage`.
- Main `index-*.js` chunk is materially smaller than the current roughly
  1.05 MB build output.
- `git diff --check` passes for touched files.

## Relevant Specs

- `.trellis/spec/admin-web/frontend/index.md`
- `.trellis/spec/admin-web/frontend/routes-forms-and-tests.md`
- `.trellis/spec/guides/index.md`
- `.trellis/spec/guides/code-reuse-thinking-guide.md`

