# Media Web Client Foundation - Handoff

Status: Active
Last updated: 2026-05-26

## Current State

MWF-010 opened the workstream. MWF-020 audited Public Client OpenAPI and the
generated TypeScript SDK. MWF-030 added the first shared Admin/Media frontend
surface boundary inside `apps/admin-web`. MWF-040 added the first browse,
search, and detail viewer routes.

The accepted direction is one shared frontend project with explicit Admin and
Media surfaces. The Media surface consumes Public Client API contracts only. It
should not reuse Admin Web data models as viewer state, and it should not make
Admin routes the playback client.

## Latest Task

- Task ID: MWF-040
- Owner: codex
- Files: `apps/admin-web/src/App.tsx`, `apps/admin-web/src/surfaces/media`,
  `apps/admin-web/src/styles.css`, workstream docs
- Validation: `cd apps/admin-web && npm run check`; `cd apps/admin-web && npm run test`;
  `cd apps/admin-web && npm run build`; boundary grep under
  `apps/admin-web/src/surfaces/media`; Playwright smoke for `/media/libraries`,
  `/media/libraries/:libraryId`, `/media/items/:itemId`, and `/media/search`.
- Status: DONE
- Review: Media Libraries, Media Library detail, URL-owned search, and Media
  Item detail now use Public Client SDK data-source methods. Library detail
  shows source evidence rather than pretending a first-class library item grid
  exists.
- Evidence: `EVIDENCE_AND_GATES.md`

## Active Task

- Task ID: MWF-050
- Owner: unassigned
- Files: `apps/admin-web/src/surfaces/media`
- Validation: focused Media Web tests plus focused server/Public API tests for
  any touched playback or `me/playback` route behavior.
- Status: READY
- Review: Do not add a browser player until the playback auth transport is
  accepted. Source selection can use `ItemDetailResponse.sources` and source
  probe facts without exposing raw server internals.
- Evidence: update `EVIDENCE_AND_GATES.md`

## Decisions Since Last Update

- Media Web is a separate surface inside the shared browser frontend, not an
  Admin route with playback controls and not a separate package for now.
- The first slice is local-media-first: libraries, browse/search, item detail,
  source selection, playback, and User Playback State.
- Public registration remains out of scope.
- Account switching starts as replacing/clearing the active connection unless
  real credential/session APIs are added in a separate lane.
- Desktop Tauri/native playback remains a follow-on.

## Blockers

- No Public Client current-principal/session summary exists.
- No first-class library-scoped item browse or Recently Added sort/feed exists.
- Browser playback needs an accepted auth transport because plain
  `<video src>` cannot send bearer headers.
- Username/password login, invitation redemption, and persistent browser
  sessions do not have accepted backend contracts yet.

## Next Recommended Action

Run MWF-050 only after choosing the browser playback auth transport. A safe
next slice is Source/Version Picker plus playback-decision preview from
`ItemDetailResponse.sources` and `getPlaybackDecision()`, with the actual player
still gated by MWF-GAP-004.
