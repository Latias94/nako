# Media Web Client Foundation - Handoff

Status: Active
Last updated: 2026-05-26

## Current State

The workstream is opened. No runtime behavior has been changed yet.

The accepted direction is a separate `apps/media-web` browser Client
Application that consumes Public Client API contracts only. It should not reuse
Admin Web data models as viewer state, and it should not make Admin Web the
playback client.

## Active Task

- Task ID: MWF-020
- Owner: unassigned
- Files: `crates/nako-api`, `sdk/typescript`,
  `docs/workstreams/media-web-client-foundation`
- Validation: `cargo test -p nako-api public_openapi -- --nocapture`;
  `cargo run -q -p nako-api --example emit-typescript-sdk -- --output sdk/typescript/src/index.ts`;
  `git diff --check`
- Status: NEEDS_CONTEXT
- Review: Public Client SDK/OpenAPI route readiness and leakage review before
  scaffolding app UI.
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

- MWF-020 must confirm whether the generated Public Client SDK has enough
  route coverage for the first app slice.
- Username/password login, invitation redemption, and persistent browser
  sessions do not have accepted backend contracts yet.

## Next Recommended Action

Run MWF-020. Build a route-to-Public-Client-contract matrix and either confirm
the first app slice can scaffold safely or split the smallest public API gap
before UI work starts.

