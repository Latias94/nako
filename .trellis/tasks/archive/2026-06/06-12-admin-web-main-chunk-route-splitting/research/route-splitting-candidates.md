# Research: route-splitting-candidates

- Query: Identify Admin Web route/page modules that should move from `App.tsx` static imports to route-level `React.lazy` dynamic imports to reduce the main Vite chunk.
- Scope: internal
- Date: 2026-06-12

## Findings

### Current build and source state

- Existing task PRD says the starting problem was an `apps/admin-web` main Vite chunk of roughly 1.05 MB with only `IncidentBundlePage` already split.
- The current `apps/admin-web/dist/assets` contents no longer match that older baseline. The present built assets include `index-Bfo-uw-L.js` at 696,407 bytes plus many route chunks, including `MediaPages-CZSukagG.js`, `StorageStagingPage-Be8pmEwR.js`, `SettingsPage-aZfqG8CA.js`, `EventsPage-DUvJ6BVw.js`, `AddonsPage-DR9-vWfQ.js`, and others.
- Current `apps/admin-web/src/App.tsx` also appears to already contain broad `React.lazy` declarations for most route page modules, not only Incident Bundle. Examples include `AcquisitionIntakePage` at `apps/admin-web/src/App.tsx:62`, `EventsPage` at `apps/admin-web/src/App.tsx:102`, `StorageStagingPage` at `apps/admin-web/src/App.tsx:157`, `SettingsPage` at `apps/admin-web/src/App.tsx:162`, `LegacyDashboard` at `apps/admin-web/src/App.tsx:167`, `IncidentBundlePage` at `apps/admin-web/src/App.tsx:172`, and the Media page exports at `apps/admin-web/src/App.tsx:177`.
- `RootLayout` wraps both media and admin outlets in `Suspense fallback={null}`, so lazy page components can render under the existing route shell without per-route `Suspense` boilerplate. Evidence: `apps/admin-web/src/App.tsx:508`, `apps/admin-web/src/App.tsx:528`, and `apps/admin-web/src/App.tsx:539`.
- The routing/search ownership pattern remains in `App.tsx`: route components still call `useRouteContext`, `useSearch`, `useNavigate`, and pass normalized `search` plus `onSearchChange` into page modules. Examples: `JobsRoute` at `apps/admin-web/src/App.tsx:548`, `CatalogBrowseRoute` at `apps/admin-web/src/App.tsx:610`, `StorageStagingRoute` at `apps/admin-web/src/App.tsx:815`, `MediaWatchRoute` at `apps/admin-web/src/App.tsx:937`; validators start with `validateMediaPageSearch` at `apps/admin-web/src/App.tsx:955`.

### Files found

- `apps/admin-web/src/App.tsx` - TanStack Router route tree, search validators, shell selection, and current lazy route page declarations.
- `apps/admin-web/src/surfaces/media/MediaPages.tsx` - large Media surface module exporting all Media route pages and browser playback helpers.
- `apps/admin-web/src/surfaces/media/MediaShell.tsx` - Media shell imported eagerly because `RootLayout` chooses it when `pathname.startsWith("/media")`.
- `apps/admin-web/src/surfaces/media/MediaSession.tsx` - Media session provider imported eagerly by `RootLayout`.
- `apps/admin-web/src/surfaces/media/mediaDataSource.ts` - Public Client / fixture Media data-source factory; currently needed by `App` default props and Media session wiring.
- `apps/admin-web/src/features/storage/StorageStagingPage.tsx` - very large storage/VFS staging route with multiple reads, mutations, tables, and remediation panels.
- `apps/admin-web/src/features/settings/SettingsPage.tsx` - very large Settings route with multiple diagnostics reads and full-replacement mutation forms.
- `apps/admin-web/src/features/events/EventsPage.tsx` - large Addon Event Delivery operator route with delivery/replay mutations.
- `apps/admin-web/src/features/addons/AddonsPage.tsx` - large Addons route with task-run retry workflow and table rendering.
- `apps/admin-web/src/features/libraries/LibraryDetailPage.tsx` - large Library detail route with profile editing and command workflow.
- `apps/admin-web/src/features/catalog/CatalogGovernanceRepairPage.tsx` - large catalog repair/review route with confirmation workflow.
- `apps/admin-web/src/features/access/AccessPage.tsx` - invitation workflow route with create/revoke mutations.
- `apps/admin-web/src/features/jobs/JobsPage.tsx` - job list and command route with table rendering.
- `apps/admin-web/src/features/items/ItemArtworkGalleryPage.tsx` - artwork gallery mutation route.
- `apps/admin-web/src/features/artwork/ManagedArtworkMaintenancePage.tsx` - managed artwork diagnostics route.
- `apps/admin-web/src/features/automation/GeneratedArtifactReviewPage.tsx` - generated artifact review mutation route.
- `apps/admin-web/src/features/items/SourceDuplicateReconciliationRoutePage.tsx` - thin route-owned wrapper that creates the feature adapter with `useMemo`.

### Code patterns

- Route-owned search stays outside page modules. Search types are imported type-only from page modules, while normalization remains in `App.tsx` (`apps/admin-web/src/App.tsx:34`, `apps/admin-web/src/App.tsx:955`, `apps/admin-web/src/App.tsx:999`, `apps/admin-web/src/App.tsx:1023`).
- Lazy imports use named export mapping through `.then((module) => ({ default: module.X }))`, which preserves existing named page exports and avoids changing page modules to default exports (`apps/admin-web/src/App.tsx:62`, `apps/admin-web/src/App.tsx:157`, `apps/admin-web/src/App.tsx:177`).
- A global outlet-level `Suspense` boundary is already present for both Admin and Media shells (`apps/admin-web/src/App.tsx:528`, `apps/admin-web/src/App.tsx:539`).
- Media is currently one very large page module with seven exported route pages and playback-specific helpers: page exports at `apps/admin-web/src/surfaces/media/MediaPages.tsx:55`, `:81`, `:141`, `:179`, `:226`, `:300`, `:333`; heavy playback helpers at `:372`, `:482`, `:1071`, `:1300`, and `:1481`.
- `SourceDuplicateReconciliationRoutePage` is already a good model for keeping route-specific adapter construction out of `App.tsx` while still lazy-loading the wrapper. It imports the adapter at `apps/admin-web/src/features/items/SourceDuplicateReconciliationRoutePage.tsx:9`, exports the wrapper at `:19`, and memoizes adapter creation at `:27`.
- Table-heavy pages import `@tanstack/react-table`; this is a good split boundary because table setup and columns are page-local. Examples: Storage at `apps/admin-web/src/features/storage/StorageStagingPage.tsx:1` and `:778`, Addons at `apps/admin-web/src/features/addons/AddonsPage.tsx:1` and `:536`, Catalog Governance Repair avoids table but has deep mutation workflow at `apps/admin-web/src/features/catalog/CatalogGovernanceRepairPage.tsx:58`.
- Mutation-heavy pages import `useMutation`, `useQuery`, and often `useQueryClient`, so lazy-loading them removes non-initial workflow code from cold route loads. Examples: Storage at `apps/admin-web/src/features/storage/StorageStagingPage.tsx:9`, Settings at `apps/admin-web/src/features/settings/SettingsPage.tsx:3`, Events at `apps/admin-web/src/features/events/EventsPage.tsx:1`, Addons at `apps/admin-web/src/features/addons/AddonsPage.tsx:9`.

### Candidate priority

#### P0 - highest value route chunks

1. `apps/admin-web/src/surfaces/media/MediaPages.tsx`
   - Current source size: 54,765 bytes / 1,810 lines. Current emitted route chunk: `MediaPages-CZSukagG.js` at 28,784 bytes.
   - Why split: this is the largest route module and includes browser playback code, progress throttling, playback ticket selection, HLS adapter detection, media lists, search, item detail, and watch UI. Most Admin operators should not pay this cost on `/overview`, `/jobs`, or `/settings`.
   - Expected benefit: high main-chunk reduction when moved out of static imports. Current build suggests it is already separated as a single chunk, but it may still be worth splitting `MediaWatchPage`/playback helpers from the lighter Media home/libraries/search pages later.
   - Risk: medium. `MediaPages.tsx` exports type-only search contracts consumed by `App.tsx`, and all Media routes currently share one dynamic import target. Splitting within the file would require moving exported page components/helpers into separate files while preserving `MediaPageSearch`, `MediaSearchRouteSearch`, and `MediaItemSearch` imports.
   - Tests: `apps/admin-web/src/surfaces/media/mediaSurface.test.tsx` has broad coverage for `/media`, `/media/libraries`, `/media/search`, `/media/items/$itemId`, and `/media/watch/$itemId`, including token redaction and browser playback behavior.

2. `apps/admin-web/src/features/storage/StorageStagingPage.tsx`
   - Current source size: 49,688 bytes / 1,434 lines. Current emitted route chunk: `StorageStagingPage-Be8pmEwR.js` at 24,640 bytes.
   - Why split: one of the largest Admin route modules, table-heavy, and owns multiple independent diagnostics and mutation workflows (`useQuery` fan-out at lines `:110`, `:114`, `:122`, `:130`, `:138`; mutations at `:230`, `:257`, `:294`).
   - Expected benefit: very high, especially for non-storage routes.
   - Risk: medium. URL search normalization must remain in `App.tsx`; route tests cover query mapping, filters, fallback, mutations, and redaction.
   - Tests: `App.test.tsx` covers `/storage/staging` search mapping, rendering, localized copy, filter updates, VFS repair context, actions, fallback, and unsafe-field rejection around `apps/admin-web/src/App.test.tsx:4046` through `:4328`.

3. `apps/admin-web/src/features/settings/SettingsPage.tsx`
   - Current source size: 46,506 bytes / 1,305 lines. Current emitted route chunk: `SettingsPage-aZfqG8CA.js` at 24,040 bytes.
   - Why split: large form/workflow page with multiple diagnostics reads, full-replacement mutation drafts, validation helpers, and confirmation flows.
   - Expected benefit: very high because Settings is not needed for normal Admin navigation until visited.
   - Risk: medium-high. The spec explicitly requires complete typed payload drafts for full-replacement `PUT` settings and live-only mutation guards. Lazy-loading should not move or alter those contracts.
   - Tests: `App.test.tsx` covers `/settings` rendering, mock fallback, zh-Hans copy, raw-cache save confirmation, runtime settings confirmation, mock mutation disablement, and redaction around `apps/admin-web/src/App.test.tsx:930` through `:1596`.

#### P1 - high value workflow chunks

4. `apps/admin-web/src/features/events/EventsPage.tsx`
   - Current source size: 31,151 bytes / 954 lines. Current emitted route chunk: `EventsPage-DUvJ6BVw.js` at 17,125 bytes.
   - Why split: route owns Addon event delivery/replay workflow, multiple data reads, and live-only mutations.
   - Expected benefit: high.
   - Risk: medium-high because redaction and mutation confirmation are central to the route contract.
   - Tests: `App.test.tsx` covers `/events` URL mapping, zh-Hans/mock disabled behavior, deliver/replay confirmation, and unsafe delivery-field redaction around `apps/admin-web/src/App.test.tsx:2352` through `:2496`.

5. `apps/admin-web/src/features/addons/AddonsPage.tsx`
   - Current source size: 30,604 bytes / 918 lines. Current emitted route chunk: `AddonsPage-DR9-vWfQ.js` at 16,396 bytes.
   - Why split: table-heavy route with Addon Task Run retry workflow and generated Admin API projection.
   - Expected benefit: high.
   - Risk: medium. The Addon Task Run spec forbids raw generated task-run details in route rendering; tests should remain async-aware under `Suspense`.
   - Tests: `App.test.tsx` covers `/addons` URL mapping, render, localization, filters, fallback, retry confirmation, and redaction around `apps/admin-web/src/App.test.tsx:2152` through `:2323`.

6. `apps/admin-web/src/features/libraries/LibraryDetailPage.tsx`
   - Current source size: 30,427 bytes / 909 lines. Current emitted route chunk: `LibraryDetailPage-Cz_zVeCl.js` at 17,040 bytes.
   - Why split: detail-only route with profile editing, command workflow, and many formatting helpers. Not needed for `/libraries` list or any shell route.
   - Expected benefit: high.
   - Risk: medium. It is linked from `LibrariesPage` and has route params; ensure `$libraryId` param wiring stays in `App.tsx`.
   - Tests: `App.test.tsx` covers `/libraries/$libraryId` rendering, profile editing, fallback, redaction, and localized copy around `apps/admin-web/src/App.test.tsx:792` through `:912`.

7. `apps/admin-web/src/features/catalog/CatalogGovernanceRepairPage.tsx`
   - Current source size: 27,052 bytes / 700 lines. Current emitted route chunk: `CatalogGovernanceRepairPage-VNpEY_Kj.js` at 15,448 bytes.
   - Why split: nested workflow route with plan reads, decision search state, and mutation confirmation; unlikely to be first screen.
   - Expected benefit: high.
   - Risk: medium-high. It depends on `$itemId`, `mapping_id`, and `decision` search state; route-owned normalization must remain unchanged.
   - Tests: `App.test.tsx` covers `/catalog/governance/$itemId` render, localized copy, URL decision state, disabled hybrid mutation, explicit confirmation, unavailable mutation error, and redaction around `apps/admin-web/src/App.test.tsx:3476` through `:3808`.

#### P2 - medium value chunks

8. `apps/admin-web/src/features/access/AccessPage.tsx`
   - Current source size: 26,168 bytes / 788 lines. Current emitted route chunk: `AccessPage-Ds73yOPl.js` at 13,515 bytes.
   - Expected benefit: medium-high; page has create/revoke invitation workflows and local form state.
   - Risk: medium-high because one-time token visibility and mutation disablement are security-sensitive.
   - Tests: `App.test.tsx` covers `/access` render, fallback, localization, create/revoke confirmation, mock disabled mutation state, and redaction around `apps/admin-web/src/App.test.tsx:1004` through `:1274`.

9. `apps/admin-web/src/features/jobs/JobsPage.tsx`
   - Current source size: 23,189 bytes / 745 lines. Current emitted route chunk: `JobsPage-8MVlgMdQ.js` at 12,705 bytes.
   - Expected benefit: medium-high; table-heavy route with live commands.
   - Risk: medium because `/jobs` is likely a common operator route and `defaultPreload: "intent"` may fetch it on hover/focus. Search normalization and redaction tests are important.
   - Tests: `App.test.tsx` covers `/jobs` search mapping, filter URL updates, zh-Hans copy, mock fallback, command actions, and redaction around `apps/admin-web/src/App.test.tsx:129` through `:335`, plus redaction at `:4345`.

10. `apps/admin-web/src/features/items/ItemArtworkGalleryPage.tsx`
    - Current source size: 21,962 bytes / 623 lines. Current emitted route chunk: `ItemArtworkGalleryPage-CivtpEXi.js` at 11,709 bytes.
    - Expected benefit: medium; nested item-specific workflow not needed for general Admin entry.
    - Risk: medium-high because select/unpublish actions are confirmation-gated and route params/search must remain stable.
    - Tests: `App.test.tsx` covers item artwork render, localization, select/unpublish confirmation, unavailable mutation, mutation-result redaction, fallback, mock disabled controls, and route redaction around `apps/admin-web/src/App.test.tsx:2973` through `:3248`.

11. `apps/admin-web/src/features/artwork/ManagedArtworkMaintenancePage.tsx`
    - Current source size: 21,044 bytes / 659 lines. Current emitted route chunk: `ManagedArtworkMaintenancePage-BOKEUpGO.js` at 11,448 bytes.
    - Expected benefit: medium; route is diagnostics-oriented and not first-load critical.
    - Risk: low-medium because current page is read-only, but query params include booleans and numeric limits.
    - Tests: `App.test.tsx` covers `/artwork/maintenance` query mapping, render, zh-Hans copy, fallback, and redaction around `apps/admin-web/src/App.test.tsx:3268` through `:3349`.

12. `apps/admin-web/src/features/automation/GeneratedArtifactReviewPage.tsx`
    - Current source size: 18,596 bytes / 491 lines. Current emitted route chunk: `GeneratedArtifactReviewPage-D4Mbbkyr.js` at 11,389 bytes.
    - Expected benefit: medium; nested review route not needed for list route or shell.
    - Risk: medium-high because decision state is URL-owned and review mutation is confirmation-gated.
    - Tests: `App.test.tsx` covers generated artifact review render, localization, URL decision state, mock fallback, mock disabled mutation, redaction, confirmation, unavailable mutation error, and result redaction around `apps/admin-web/src/App.test.tsx:1863` through `:2121`.

#### P3 - lower value or keep eager unless main chunk remains too large

13. `apps/admin-web/src/features/overview/OverviewPage.tsx`
    - Current source size: 17,881 bytes / 540 lines. Current emitted route chunk: `OverviewPage-BtJiGp0v.js` at 10,077 bytes.
    - Expected benefit: medium but tradeoff is worse than other pages because `/` redirects to `/overview`, so this is effectively the initial page for most Admin sessions.
    - Recommendation: keep lazy only if the shell should appear before overview data code loads; otherwise consider making Overview eager if route chunk waterfalls hurt perceived first load.

14. `apps/admin-web/src/features/items/ItemDetailPage.tsx`
    - Current source size: 15,094 bytes / 432 lines. Current emitted route chunk: `ItemDetailPage-BlYCKoTN.js` at 8,989 bytes.
    - Expected benefit: medium-low. Good nested detail split, but not as large as P0/P1.
    - Risk: low-medium; has links into playback support and artwork gallery routes.

15. `apps/admin-web/src/features/items/SourceDuplicateReconciliationRoutePage.tsx` plus `SourceDuplicateReconciliationPage.tsx` and adapter
    - Current wrapper source size: 1,335 bytes / 45 lines; page source size: 14,131 bytes / 414 lines; adapter source size: 2,181 bytes / 62 lines. Current emitted route chunk: `SourceDuplicateReconciliationRoutePage-DwLKpOXK.js` at 8,454 bytes.
    - Expected benefit: medium-low, but the pattern is architecturally clean because `App.tsx` imports only the lazy wrapper and type-only search, while adapter construction lives beside the feature.
    - Risk: low-medium. Preserve `useMemo` adapter construction and localized unavailable messages.

16. `AcquisitionIntakePage`, `GeneratedArtifactsPage`, `CatalogBrowsePage`, `CatalogGovernancePage`, `PlaybackSupportPage`, `PlaybackSessionsPage`, and `LibrariesPage`
    - Current emitted chunks range from 3,698 to 9,459 bytes.
    - Expected benefit: low to medium individually. Still useful as route chunks after high-priority pages are split, but these are not the main source of the 1.05 MB baseline.
    - Recommendation: split them opportunistically with the same lazy pattern if broad consistency is desired. Prioritize only after P0/P1 unless tests already cover the route heavily and implementation is mechanical.

### Route and test risks

- `React.lazy` introduces asynchronous route rendering under `Suspense`. Route tests that previously assumed immediate page content must use `findBy...`/`waitFor`, which existing Admin Web tests already commonly do.
- `Suspense fallback={null}` means there is no visible loading state for page-module fetch. That keeps visual behavior minimal but can make test failures look like absent content until async waits resolve.
- Type-only search imports from page modules are safe only while TypeScript erases them. Do not accidentally convert search-type imports to runtime imports, or the page module will re-enter the main chunk.
- Media currently lazy-loads one shared `MediaPages.tsx` module for all Media route pages. This removes Media from the Admin main path, but it does not create separate chunks for `/media/libraries` vs `/media/watch`. The next high-value Media improvement is file-level separation of watch/playback code from browse/search pages.
- `SourceDuplicateReconciliationRoutePage` shows that route-owned wrappers can reduce `App.tsx` runtime imports when a page needs feature adapter construction. This pattern should be reused for future routes with route-specific adapter factories instead of importing adapter factories directly in `App.tsx`.
- `defaultPreload: "intent"` in `createAppRouter` (`apps/admin-web/src/App.tsx:466`) means lazy chunks may preload on link intent. This is desirable for navigation smoothness but can mask bundle-size changes during manual browsing; build output is the source of truth.

### Suggested implementation order

1. Confirm current source and dist baseline before changing anything. The present `App.tsx` and `dist/assets` already show broad lazy splitting, so the remaining implementation may be verification or cleanup rather than initial conversion.
2. If working from a branch that still has static imports, first convert P0/P1 modules with the existing named-export lazy pattern and the global `Suspense` boundary: Media pages, Storage, Settings, Events, Addons, Library detail, and Catalog Governance repair.
3. Convert P2 workflow pages next: Access, Jobs, Item Artwork Gallery, Managed Artwork Maintenance, and Generated Artifact Review.
4. Convert P3 pages only for consistency or if the main chunk remains too large after P0/P2.
5. After broad lazy conversion, consider Media internal splitting: move `MediaWatchPage`, `MediaBrowserPlayer`, `MediaVideoElement`, ticket selection, progress flushing, and HLS adapter detection into a watch-specific module so browsing `/media/libraries` does not fetch the whole playback stack.

### Verification targets

- `npm run check --prefix apps/admin-web`
- `npm run test --prefix apps/admin-web`
- `npm run build --prefix apps/admin-web`
- Compare `apps/admin-web/dist/assets/index-*.js` before/after and confirm route chunks are emitted for the selected page modules.
- Focused test areas: `apps/admin-web/src/App.test.tsx` and `apps/admin-web/src/surfaces/media/mediaSurface.test.tsx`.

### External references

- Package versions observed locally in `apps/admin-web/package.json`: React `19.2.6`, `@tanstack/react-router` `^1.170.8`, `@tanstack/react-query` `^5.100.14`, `@tanstack/react-table` `^8.21.3`, Vite `8.0.13`, TypeScript `6.0.3`.
- No web documentation lookup was needed for this internal codebase research.

### Related specs

- `.trellis/tasks/06-12-admin-web-main-chunk-route-splitting/prd.md`
- `.trellis/spec/admin-web/frontend/index.md`
- `.trellis/spec/admin-web/frontend/routes-forms-and-tests.md`
- `.trellis/spec/guides/index.md`
- `.trellis/spec/guides/code-reuse-thinking-guide.md`

## Caveats / Not Found

- No active Trellis task was set according to `python3 ./.trellis/scripts/task.py current --source`, but the user supplied the exact task output path. This research was written only to that requested task's `research/` directory.
- The workspace appears to have changed from the PRD's baseline: current `App.tsx` already has broad lazy route declarations, and current `dist/assets` already has many route chunks with a smaller `index-Bfo-uw-L.js` at 696,407 bytes. This may reflect earlier implementation work or a regenerated build while this research was being prepared.
- I did not run `npm run build`, `npm run check`, or tests because this was a read-only research request. Dist observations are from the existing `apps/admin-web/dist/assets` files.
- I did not inspect or copy any `repo-ref` code.
