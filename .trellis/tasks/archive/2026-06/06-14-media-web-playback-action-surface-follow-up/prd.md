# Media Web Playback Action Surface Follow-up

## Goal

Continue the Media Web playback-state work by making Continue Watching a useful action surface, not only a Resume list.

## What I Already Know

- The last shipped slice added item-detail Resume and Start over actions.
- Resume uses saved playback-state `source_id` continuity.
- Start over clears item detail progress through the existing Media Web data-source mutation.
- Watch already has a browser-player Start over action with a different meaning from the playback-state Start over button.
- Current scope should stay frontend-only unless repo inspection reveals a small API gap.
- Home Continue Watching rows currently expose only Resume.
- The Media Web data source already exposes `setUserWatchedState`, and fixture mode already removes entries from Continue Watching when resume state is cleared.
- Admin Web specs require route/data-source boundaries, no page-level fetch, and redaction-safe rendering.

## Assumptions

- The next useful slice is adding Continue Watching row-level Start over/remove behavior in `apps/admin-web`, not a backend or SDK change.
- Media Web should continue using URL-owned route/search state and existing data-source boundaries.
- We should preserve current fixture/live connection behavior and redaction rules.

## Open Questions

- Confirm whether Continue Watching row-level Start over is the desired MVP.

## Requirements (Evolving)

- Keep backend, SDK, database, and Public Client API unchanged unless a blocking gap is discovered.
- Preserve sensitive-data redaction: no ticket tokens, stream URLs, bearer tokens, fingerprints, raw paths, or source internals in rendered text.
- Add a row-level action to Home Continue Watching entries that clears the saved resume state through `setUserWatchedState`.
- The action must use the Continue Watching entry's saved `source_id` when available.
- Fixture mode must update the visible Continue Watching list after the mutation.

## Acceptance Criteria (Evolving)

- [x] Continue Watching rows expose a clear/start-over action next to Resume.
- [x] The action calls `setUserWatchedState` with `watched: false`, `position_ms: 0`, duration from the entry state, and saved `source_id`.
- [x] Fixture mode removes the row from Continue Watching after the action succeeds.
- [x] Existing media playback, resume, Continue Watching, and redaction tests still pass.
- [x] Admin Web check and build pass.

## Definition of Done

- Tests added or updated for the chosen user-facing behavior.
- `npm run check --prefix apps/admin-web` passes.
- Focused Media Web tests pass.
- `npm run test --prefix apps/admin-web` passes when risk requires it.
- `npm run build --prefix apps/admin-web` passes when route/page code changes.
- Trellis task validates and is archived after completion.

## Out of Scope

- Backend playback planner, streaming, database, SDK, or Public Client API changes.
- A new global frontend state manager.
- Broad product redesign of Media Web.
- Watch-page browser-player Start over behavior.
- Item-detail playback-state behavior already shipped in the previous slice.

## Technical Notes

- Files inspected:
  - `apps/admin-web/src/surfaces/media/MediaPages.tsx`
  - `apps/admin-web/src/surfaces/media/MediaItemShared.tsx`
  - `apps/admin-web/src/surfaces/media/mediaDataSource.ts`
  - `apps/admin-web/src/surfaces/media/mediaSurface.test.tsx`
  - `.trellis/spec/admin-web/frontend/index.md`
  - `.trellis/spec/admin-web/frontend/routes-forms-and-tests.md`

## Feasible Approaches

### Approach A: Row-level Start over on Continue Watching (Recommended)

- Add a button beside each Continue Watching Resume link.
- Call `setUserWatchedState` through the Media Web data source.
- Keep state local to `MediaHomePage` by refreshing/removing the affected row after mutation.

Pros: smallest useful slice, consistent with item detail, no API changes.
Cons: Home page gets a small mutation state path.

### Approach B: Link Continue Watching users to item detail for all actions

- Keep Home read-only and rely on item detail Start over.

Pros: minimal code.
Cons: extra navigation for a common cleanup action and weaker Home UX.

## Decision (ADR-lite)

**Context**: Item detail can now clear resume state, but Home Continue Watching still only links to Resume.

**Decision**: Prefer Approach A unless the user wants Home to remain read-only.

**Consequences**: Continue Watching becomes a true action rail while preserving existing Public Client mutation boundaries.
