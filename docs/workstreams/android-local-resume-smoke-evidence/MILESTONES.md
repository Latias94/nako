# Android Local Resume Smoke Evidence - Milestones

Status: Closed
Last updated: 2026-05-19

## M0 - Boundary Freeze

Exit criteria:

- Workstream docs exist and agree on the local-only resume boundary.
- Follow-ons for server-authoritative **User Playback State** remain out of
  scope.

Status: Complete.

## M1 - Device-Local Resume Smoke Slice

Exit criteria:

- Debug-only fixture seeding can write an optional local playback position.
- Smoke orchestration resolves the fixture Media Item and Media Source ids from
  the running server.
- `profile-with-media` asserts local resume UI text and forbids
  server-authoritative resume wording on the source picker surface.
- Focused test, focused smoke, and diff hygiene gates pass.

Status: Complete.

## M2 - Closeout

Exit criteria:

- Evidence docs reference the final command output paths.
- TODO, DESIGN, HANDOFF, and WORKSTREAM status are closed.
- Remaining CI, golden screenshot, and deeper playback validation work is
  explicitly split as follow-on scope.

Status: Complete.
