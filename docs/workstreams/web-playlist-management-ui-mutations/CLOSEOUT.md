# Web Playlist Management UI Mutations - Closeout

Status: closed
Closed: 2026-05-29

## Closeout Claim

This lane is complete for web playlist management mutations. The Media Web
playlist surface can now create, rename, delete, add items, remove items, and
reorder items through the Public Client boundary with truthful fixture fallback
and route-owned playlist/view state.

## Delivered

- Public Client web data-source methods for playlist create, update, delete,
  add item, remove item, and reorder.
- TanStack Query mutation hooks for those operations with playlist list/items
  invalidation and deleted playlist item-cache removal.
- `/media/my-list` create, rename, delete, remove item, and reorder controls.
- Narrow add-to-playlist entry points from media detail and browse cards.
- Explicit up/down reorder controls before richer drag-and-drop.
- Stale-version reorder recovery that refetches the current item order.
- Route-owned `playlist` and `view` state preservation.
- Fixture mutation feedback that does not claim persistence.
- Route, state, data-source, hook, bundle budget, and browser smoke evidence.

## Verification

Passed during the lane:

```bash
npm --prefix web run test -- src/test/data-source-contracts.test.ts src/test/use-media-contracts.test.tsx
npm --prefix web run test -- src/test/route-contracts.test.tsx src/test/route-state-contracts.test.tsx
npm --prefix web run test
npm --prefix web run check
npm --prefix web run build:budget
python -m json.tool docs/workstreams/web-playlist-management-ui-mutations/WORKSTREAM.json
git diff --check -- web docs/workstreams/web-playlist-management-ui-mutations
```

Browser smoke passed for:

- desktop `/media/my-list?view=list`;
- mobile `390x844` `/media/my-list?playlist=fixture-favorites&view=grid`;
- mobile `/media` add-to-playlist menu and fixture non-persistence feedback.

## Review Result

### Workstream Compliance

- Blocking: none.
- All WPMU tasks are complete.
- The shipped UI stays on the Public Client data-source and TanStack Query hook
  boundary.
- Fixture mode reports mutation non-persistence instead of pretending writes
  succeeded.
- Route/state tests cover playlist selection/view state, create, rename,
  delete, remove item, add item, reorder, and stale-version reorder recovery.

### Code Quality

- Blocking: none.
- Playlist mutation code is localized in `web/lib/use-media.ts`,
  `web/src/api/public/media-data-source.ts`, and media feature components.
- Tests exercise behavior through route/data-source/hook seams rather than
  private component implementation.
- Reorder starts with accessible explicit controls; drag-and-drop remains out
  of scope.

### Missing Gates

- None for the shipped web/docs scope.
- Rust/API/SDK gates were not rerun in closeout because WPMU-030 through
  WPMU-060 changed only `web/` and workstream docs, and WPMU-020 already
  validated the generated SDK-facing mutation boundary.

## Follow-Ons

- Broader add-to-playlist affordances across search, library, person, and
  collection result surfaces if product usage justifies them.
- Rich drag-and-drop reorder after the explicit accessible controls prove
  insufficient.
- Shared/public playlists, invites, and collaboration.
- Smart playlists and recommendation-generated lists.
- Offline sync conflict resolution.
- Mobile/Tauri playlist management surfaces after the web UX remains stable.

## Residual Risk

The lane intentionally avoids shared playlists, smart lists, collaboration, and
offline sync. The current reorder UX is explicit and accessible, but not a rich
drag-and-drop interaction. Those are product follow-ons rather than blockers for
closing this Public Client-backed web mutation lane.

## Evidence Anchors

- `docs/workstreams/web-playlist-management-ui-mutations/EVIDENCE_AND_GATES.md`
- `docs/workstreams/web-playlist-management-ui-mutations/TODO.md`
- `web/src/features/media/my-list-page.tsx`
- `web/src/features/media/add-to-playlist-button.tsx`
- `web/src/api/public/media-data-source.ts`
- `web/lib/use-media.ts`
- `web/src/test/route-state-contracts.test.tsx`
- `web/src/test/data-source-contracts.test.ts`
- `web/src/test/use-media-contracts.test.tsx`
