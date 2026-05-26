# Media Web Client Foundation - Handoff

Status: Active
Last updated: 2026-05-26

## Current State

MWF-010 opened the workstream. MWF-020 audited Public Client OpenAPI and the
generated TypeScript SDK. MWF-030 added the first shared Admin/Media frontend
surface boundary inside `apps/admin-web`.

The accepted direction is one shared frontend project with explicit Admin and
Media surfaces. The Media surface consumes Public Client API contracts only. It
should not reuse Admin Web data models as viewer state, and it should not make
Admin routes the playback client.

## Latest Task

- Task ID: MWF-030
- Owner: codex
- Files: `apps/admin-web/package.json`, `apps/admin-web/src/App.tsx`,
  `apps/admin-web/src/components/layout/AdminShell.tsx`,
  `apps/admin-web/src/surfaces/media`, `apps/admin-web/src/styles.css`,
  workstream docs
- Validation: `cd apps/admin-web && npm run check`; `cd apps/admin-web && npm run build`;
  `cd apps/admin-web && npm run test`; boundary grep under
  `apps/admin-web/src/surfaces/media`; Playwright smoke for `/media` and
  `/overview`.
- Status: DONE
- Review: Media and Admin now coexist through route namespaces and symmetric
  surface switchers. Media uses generated Public Client SDK data sources and
  does not import Admin API runtime modules.
- Evidence: `EVIDENCE_AND_GATES.md`

## Active Task

- Task ID: MWF-040
- Owner: unassigned
- Files: `apps/admin-web/src/surfaces/media`
- Validation: `cd apps/admin-web && npm run check && npm run test`; browser
  smoke for `/media/libraries`, `/media/libraries/:libraryId`,
  `/media/search`, and `/media/items/:itemId`.
- Status: READY
- Review: Implement browse/search/detail only from accepted Public Client API
  routes. Split the smallest Public Client API gap before adding fake viewer
  contracts.
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

Run MWF-040: add Media Library detail, URL-owned search/browse state, and Media
Item detail using Public Client API routes. Keep the current fixture/live
boundary, and split MWF-GAP-002 before implementing a real library-scoped item
grid. Do not start the player until MWF-GAP-004 has an accepted auth transport.
