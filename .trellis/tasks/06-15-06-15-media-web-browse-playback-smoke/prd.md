# Media Web Browse To Playback Smoke

## Goal

Close the first Media Web user loop from browsing a library item to opening the
watch player, issuing a browser playback ticket, writing progress, and returning
to Continue Watching without exposing unsafe playback internals.

## What I Already Know

- Media Web now has a Library detail `Library items` panel backed by
  `listLibraryItems`.
- Existing Media Web routes include library browse, item detail, watch/player,
  Continue Watching, playback decision preview, browser playback tickets, and
  user playback state writes.
- The next product risk is not another browse control; it is proving the
  browse -> item detail -> watch -> progress -> Continue Watching loop.
- Media Web error rendering must stay redaction-safe: no raw stream URLs,
  bearer tokens, ticket tokens, source locators, paths, fingerprints, or backend
  internals in DOM text.

## Assumptions

- This task should be an Admin Web / Media Web validation slice, not a backend
  API or SDK contract change unless repo inspection proves a blocker.
- The MVP should prefer route/test coverage and small UX fixes over broad
  player redesign.
- Fixture mode is acceptable as the first smoke target; live mode behavior is
  covered through data-source/client tests if needed.

## Open Questions

- None.

## Requirements (Evolving)

- Add a Media Web route test that starts from
  `/media/libraries/library-anime?limit=1&offset=0`.
- The smoke opens the `Library items` card, verifies item detail source/playback
  controls, opens the watch route, simulates browser playback progress, returns
  home, and verifies Continue Watching reflects the new progress/source.
- Keep the loop URL-owned where routes already own source/item state.
- Preserve existing safe error behavior for playback ticket and player failures.
- Add only small UX affordances needed for this loop; avoid a player redesign.
- Avoid adding new routes unless the existing routes cannot represent the loop.
- Confirmed MVP scope: route-level smoke only. If a missing link appears, make
  the smallest route/test fix and keep UI redesign out of scope.

## Acceptance Criteria (Evolving)

- [x] A route test starts from the Library detail `Library items` panel, opens
      the Pilot item, opens Watch, records progress, returns Home, and observes
      Continue Watching state for the selected source.
- [x] The smoke proves browser playback ticket and player UI do not render raw
      playback URLs, ticket tokens, source paths, bearer tokens, or fingerprints.
- [x] Existing focused Media Web tests pass.
- [x] `npm run check --prefix apps/admin-web` passes.
- [x] `npm run build --prefix apps/admin-web` passes if route/page code changes.

## Definition of Done

- Tests added/updated for the chosen smoke path.
- TypeScript check and focused Media Web tests pass.
- Docs/spec updated only if the task establishes a durable route/player
  contract not already captured.
- Trellis context files are curated before implementation.

## Out of Scope

- Backend playback planning or transcode runtime changes.
- New player library selection or visual redesign.
- Full browser/device compatibility matrix.
- Native/TV/casting client behavior.
- New Media Web routes.
- Live-server E2E; fixture mode is the MVP smoke target.

## Technical Notes

- Relevant specs:
  - `.trellis/spec/admin-web/frontend/index.md`
  - `.trellis/spec/admin-web/frontend/routes-forms-and-tests.md`
  - `.trellis/spec/guides/index.md`
- Inspected files:
  - `apps/admin-web/src/surfaces/media/MediaPages.tsx`
  - `apps/admin-web/src/surfaces/media/MediaWatchPage.tsx`
  - `apps/admin-web/src/surfaces/media/MediaItemDetailPage.tsx`
  - `apps/admin-web/src/surfaces/media/MediaItemShared.tsx`
  - `apps/admin-web/src/surfaces/media/mediaSurface.test.tsx`
  - `apps/admin-web/src/surfaces/media/mediaDataSource.ts`
- Existing coverage already tests ticket redaction, retry behavior, source
  change, progress writes, pause/end flushing, and Continue Watching refresh
  from direct watch-route entry. The gap is starting from library browse.
