# Media Web Continue Watching Refresh Loop

## Goal

Close the Media Web user playback state loop so progress and watched updates written from the watch page are reflected when the operator returns to the `/media` Continue Watching surface. This keeps fixture development behavior aligned with the live Public Client API contract without changing backend routes, SDK types, or storage.

## Requirements

- Continue Watching must use the latest User Playback State available from the Media Web data source.
- Fixture mode must persist playback progress and watched mutations for the lifetime of one media session.
- Progress writes from `/media/watch/$itemId` must update the fixture Continue Watching row, including percent, resume position, source continuity, and updated timestamps.
- Watched writes must remove watched items from the active Continue Watching list.
- Returning to `/media` after progress or watched writes must show the refreshed active Continue Watching state.
- Keep live mode as a simple Public Client API read-after-write model; no backend, SDK, or API contract changes.
- Preserve existing Media Web redaction rules: do not render browser-ticket tokens, stream URLs, bearer tokens, source fingerprints, raw paths, or source internals.

## Acceptance Criteria

- [ ] Fixture mode progress update on the watch page changes the `/media` Continue Watching percent and resume link source when returning to Media home.
- [ ] Fixture mode ended/watched update removes the item from active Continue Watching when returning to Media home.
- [ ] Existing auto-resume, progress throttling, pause flush, watched-state, and source continuity tests still pass.
- [ ] Media Web rendered text still excludes ticket tokens, stream paths, bearer tokens, fingerprints, and raw paths.
- [ ] Admin Web check, focused Media Web tests, full Admin Web tests, and build pass.

## Definition of Done

- Tests added or updated for the new refresh behavior.
- `npm run check --prefix apps/admin-web` passes.
- Focused Media Web Vitest files pass.
- `npm run test --prefix apps/admin-web` passes.
- `npm run build --prefix apps/admin-web` passes.
- Trellis task validates, is archived after completion, and the journal records the session outcome.

## Technical Approach

Use the existing Media Web data-source boundary. The live data source already writes through the generated Public Client SDK and reads Continue Watching through the same SDK, so a route remount can observe server state without new client cache plumbing.

For fixture mode, make `createFixtureMediaDataSource()` hold a small in-memory copy of User Playback State. `updateUserPlaybackProgress()` and `setUserWatchedState()` will update that copy and return the updated response. `getUserPlaybackState()` and `listContinueWatching()` will derive responses from the same copy. Watched rows are filtered out of active Continue Watching.

Keep the page layer unchanged unless tests reveal a missing route reload dependency. This preserves route ownership and avoids adding a global event bus or page-level fetch.

## Decision (ADR-lite)

**Context**: The previous playback-state work writes progress and watched state, but fixture Continue Watching still returns a fixed static snapshot. That makes local development and route tests understate the read-after-write behavior expected from the live Public Client API.

**Decision**: Store fixture playback state inside the fixture data source instance and derive Continue Watching from it.

**Consequences**: Fixture behavior now mirrors live session semantics more closely while remaining deterministic per test/data-source instance. This does not solve cross-tab live invalidation or background polling; those are product-client concerns for a later broader client cache strategy.

## Out of Scope

- Backend route, database, SDK, or Public Client API changes.
- Cross-tab, push, polling, or long-lived live cache invalidation.
- A global Media Web state manager.
- New controls on the Media home page.

## Technical Notes

- Specs read:
  - `.trellis/spec/admin-web/frontend/index.md`
  - `.trellis/spec/admin-web/frontend/routes-forms-and-tests.md`
  - `.trellis/spec/guides/index.md`
  - `.trellis/spec/guides/code-reuse-thinking-guide.md`
  - `.trellis/spec/guides/cross-layer-thinking-guide.md`
- Domain glossary read: `CONTEXT.md`.
- Relevant behavior inspected:
  - Media Web session lazy data source wrapper.
  - Fixture and live Media Web data sources.
  - Media home Continue Watching rendering.
  - Watch page progress, pause flush, ended watched-state, and auto-resume tests.
