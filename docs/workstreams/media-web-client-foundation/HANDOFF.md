# Media Web Client Foundation - Handoff

Status: Active
Last updated: 2026-05-26

## Current State

MWF-010 opened the workstream. MWF-020 audited Public Client OpenAPI and the
generated TypeScript SDK. MWF-030 added the first shared Admin/Media frontend
surface boundary inside `apps/admin-web`. MWF-040 added the first browse,
search, and detail viewer routes. MWF-050 added source/version selection,
playback decision preview, a safe watch shell, and User Playback State writes.

The accepted direction is one shared frontend project with explicit Admin and
Media surfaces. The Media surface consumes Public Client API contracts only. It
should not reuse Admin Web data models as viewer state, and it should not make
Admin routes the playback client.

## Latest Task

- Task ID: MWF-050
- Owner: codex
- Files: `apps/admin-web/src/App.tsx`, `apps/admin-web/src/surfaces/media`,
  `apps/admin-web/src/styles.css`, workstream docs
- Validation: `cd apps/admin-web && npm run test -- mediaSurface.test.tsx mediaDataSource.test.ts`;
  `cd apps/admin-web && npm run test -- App.test.tsx mediaSurface.test.tsx mediaDataSource.test.ts`;
  `cd apps/admin-web && npm run check`; `cd apps/admin-web && npm run test`;
  `cd apps/admin-web && npm run build`; boundary grep under
  `apps/admin-web/src/surfaces/media`; Playwright smoke for source switching,
  watch shell, and watched-state writes.
- Status: DONE_WITH_CONCERNS
- Review: Media Item detail and `/media/watch/:itemId` now share
  Source/Version Picker state via `source_id`, call playback decisions through
  the generated Public Client SDK, and write watched/unwatched state through
  `/users/me/playback-state`. The watch route does not render a fake `<video>`
  or mint stream URLs while browser playback auth transport is unresolved.
- Evidence: `EVIDENCE_AND_GATES.md`

## Active Task

- Task ID: MWF-060
- Owner: planner
- Files: `docs/workstreams/media-web-client-foundation`
- Validation: final package-local Media Web gates, `git diff --check`, JSON
  validation, browser desktop/mobile smoke review, and closeout docs.
- Status: READY
- Review: Close the lane with named follow-ons instead of broadening this
  foundation slice. Keep real browser playback gated on MWF-GAP-004.
- Evidence: update `EVIDENCE_AND_GATES.md`

## Decisions Since Last Update

- Media Web is a separate surface inside the shared browser frontend, not an
  Admin route with playback controls and not a separate package for now.
- The first slice is local-media-first: libraries, browse/search, item detail,
  source selection, safe playback shell, and User Playback State.
- Public registration remains out of scope.
- Account switching starts as replacing/clearing the active connection unless
  real credential/session APIs are added in a separate lane.
- Desktop Tauri/native playback remains a follow-on.

## Blockers

- No Public Client current-principal/session summary exists.
- No first-class library-scoped item browse or Recently Added sort/feed exists.
- Browser playback needs an accepted auth transport because plain
  `<video src>` cannot send bearer headers.
- MWF-050 deliberately left real browser media playback as a follow-on rather
  than pretending bearer-only stream URLs are safe.
- Username/password login, invitation redemption, and persistent browser
  sessions do not have accepted backend contracts yet.

## Next Recommended Action

Run MWF-060 closeout. Verify final gates, record the watch-shell evidence, and
split follow-ons for Management Context Links, credential/session UX,
invitation onboarding, browser playback auth transport, desktop Tauri/native
playback, and recommendations.
