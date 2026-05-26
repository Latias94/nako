# Media Web Client Foundation - Handoff

Status: Active
Last updated: 2026-05-26

## Current State

MWF-010 opened the workstream. MWF-020 audited Public Client OpenAPI and the
generated TypeScript SDK. No runtime behavior has been changed yet.

The accepted direction is a separate `apps/media-web` browser Client
Application that consumes Public Client API contracts only. It should not reuse
Admin Web data models as viewer state, and it should not make Admin Web the
playback client.

## Latest Task

- Task ID: MWF-020
- Owner: codex
- Files: `crates/nako-api`, `sdk/typescript`,
  `docs/workstreams/media-web-client-foundation`
- Validation: `cargo test -p nako-api public_openapi -- --nocapture`;
  `cargo run -q -p nako-api --example emit-typescript-sdk -- --output sdk/typescript/src/index.ts`;
  `git diff --check`
- Status: DONE_WITH_CONCERNS
- Review: `ROUTE_API_READINESS.md` records the route matrix and blocking or
  deferred gaps.
- Evidence: `EVIDENCE_AND_GATES.md`

## Active Task

- Task ID: MWF-030
- Owner: unassigned
- Files: `apps/media-web`, `package.json`
- Validation: `cd apps/media-web && npm run check && npm run test`; boundary
  grep for Admin API dependencies.
- Status: NEEDS_CONTEXT
- Review: Scaffold only the accepted boundary from `ROUTE_API_READINESS.md`.
- Evidence: update `EVIDENCE_AND_GATES.md`

## Decisions Since Last Update

- Media Web is a separate browser Client Application, not an Admin Web playback
  route.
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

Run MWF-030 only inside the accepted scaffold boundary: connect shell,
Public Client SDK data-source boundary, fixture/live separation, and routes
that use existing contracts. Split MWF-GAP-002 before building a real library
item grid and MWF-GAP-004 before a real browser player.
