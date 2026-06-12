# Evidence

## Changes

- Converted Admin Web route page value imports in `apps/admin-web/src/App.tsx`
  to route-level `React.lazy` declarations while keeping route search, params,
  context, and navigation ownership in `App.tsx`.
- Moved source duplicate reconciliation adapter creation into a lazy
  feature-owned route wrapper.
- Deferred full i18n message catalog loading through dynamic import while
  preserving typed `MessageId` / `AdminLocale` usage and avoiding stale-catalog
  rendering on locale switches.
- Deferred the default Media Web data source through a lazy proxy so the Admin
  shell does not statically import Public Client SDK and fixture data.
- Added the Admin Web route-level bundle-splitting convention to the frontend
  Trellis spec.

## Build Result

- Previous observed main chunk after Incident Bundle-only split:
  `index-BaRVXfkL.js` at about `1,049.33 kB`.
- Intermediate route-page split result: main chunk about `696.40 kB`.
- Final result after route, i18n catalog, and Media data-source splitting:
  `index-Coquxbr4.js` at `482.88 kB` / gzip `139.65 kB`.
- New supporting chunks include `messages-WXcQIiVn.js`,
  `mediaDataSource-DwkonjCs.js`, and per-route page chunks.

## Verification

- `npm run check --prefix apps/admin-web`
- `npm run test --prefix apps/admin-web`
- `npm run build --prefix apps/admin-web`
- `git diff --check`
- `python ./.trellis/scripts/task.py validate 06-12-admin-web-main-chunk-route-splitting`

