# Media Web Client Foundation - Handoff

Status: Closed
Last updated: 2026-05-26

## Current State

MWF-010 opened the workstream. MWF-020 audited Public Client OpenAPI and the
generated TypeScript SDK. MWF-030 added the first shared Admin/Media frontend
surface boundary inside `apps/admin-web`. MWF-040 added the first browse,
search, and detail viewer routes. MWF-050 added source/version selection,
playback decision preview, a safe watch shell, and User Playback State writes.
MWF-060 closed the lane and split follow-ons.

The accepted direction is one shared frontend project with explicit Admin and
Media surfaces. The Media surface consumes Public Client API contracts only. It
should not reuse Admin Web data models as viewer state, and it should not make
Admin routes the playback client.

## Latest Task

- Task ID: MWF-060
- Owner: codex
- Files: `docs/workstreams/media-web-client-foundation`
- Validation: final package-local Media Web gates, JSON validation,
  `git diff --check`, boundary grep, and desktop/mobile browser smoke review.
- Status: DONE
- Review: The lane is closed in `CLOSEOUT.md`; remaining work is split in
  `FOLLOW_ON_SPLIT.md`. Real browser playback remains gated on MWF-GAP-004.
- Evidence: `EVIDENCE_AND_GATES.md`, `CLOSEOUT.md`, `FOLLOW_ON_SPLIT.md`

## Active Task

None. This workstream is closed.

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

Start a new bounded lane from `FOLLOW_ON_SPLIT.md`. Recommended first lane:
Browser Playback Auth Transport.
