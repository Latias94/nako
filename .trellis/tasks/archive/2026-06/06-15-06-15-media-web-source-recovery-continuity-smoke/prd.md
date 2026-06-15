# Media Web Source Recovery Continuity Smoke

## Goal

Prove that Media Web playback recovery paths keep the selected source consistent after a browser path failure, next-path recovery, progress write, and return to Continue Watching.

## What I Already Know

- Existing Media Web tests cover ticket retry error copy and redaction.
- Existing tests cover trying the next browser ticket URL after a playback path fails.
- Existing tests cover source-specific candidate reset after source changes.
- Existing tests cover direct progress writes and Continue Watching source continuity.
- The remaining gap is a user-level recovery loop: fail a playback path, recover to the next path, then write progress and verify Continue Watching still resumes the same selected source.
- Media Web playback tests must not render raw stream URLs, ticket tokens, `/sources/`, bearer tokens, fingerprints, or backend internals.

## Assumptions

- This is an Admin Web route-level test slice, not a backend/API/SDK change.
- Fixture mode and existing multi-path ticket helpers are enough to prove the MVP.
- No UI redesign or new recovery controls are needed unless the smoke exposes a real gap.

## Open Questions

- None.

## Requirements

- Add a focused Media Web smoke that starts at `/media/watch/item-episode-1?source_id=source-episode-1-alt`.
- Use a multi-path browser playback ticket for the selected alternate source.
- Simulate the first path failing and recover with `Try next path`.
- After recovery, simulate playback progress and assert the progress write uses `source_id: "source-episode-1-alt"`.
- Navigate Home and assert Continue Watching Resume links back to `/media/watch/item-episode-1?source_id=source-episode-1-alt`.
- Preserve redaction assertions for ticket/path/source internals.

## Acceptance Criteria

- [x] A route test covers first path failure -> next path recovery -> progress write -> Home Continue Watching Resume.
- [x] The recovered player uses the second path from the same alternate source ticket.
- [x] `updateUserPlaybackProgress` is called with `source_id: "source-episode-1-alt"` after recovery.
- [x] Home Resume links to `/media/watch/item-episode-1?source_id=source-episode-1-alt`.
- [x] Unsafe ticket/path/source internals are not rendered.
- [x] `npm run test --prefix apps/admin-web -- src/surfaces/media/mediaSurface.test.tsx` passes.
- [x] `npm run check --prefix apps/admin-web` passes.

## Definition of Done

- Focused route smoke added to `mediaSurface.test.tsx`.
- No production behavior changes unless a real continuity bug is found.
- Docs/spec updated only if a new durable route/player contract is established.
- Trellis context files configured before implementation.

## Technical Approach

Extend the existing Media Web route test coverage with a narrow smoke near the current browser path recovery tests. Reuse `multiplePathTicket`, `setMediaTiming`, and fixture data-source spies. The smoke should compose existing behaviors rather than create new helper APIs.

## Decision (ADR-lite)

**Context**: Recovery behavior and source continuity are covered separately, but user confidence depends on their composition.

**Decision**: Validate the composition with a route smoke, keeping product behavior unchanged.

**Consequences**: This improves confidence in playback recovery without committing to broader retry UX changes. Future source-aware recovery UI can build on the same route-owned source model.

## Out of Scope

- New recovery controls or player redesign.
- Backend playback ticket or path selection changes.
- hls.js behavior beyond existing tests.
- Public Client SDK changes.
- Live-server E2E or device compatibility matrix.

## Technical Notes

- Relevant spec: `.trellis/spec/admin-web/frontend/routes-forms-and-tests.md`.
- Relevant files inspected:
  - `apps/admin-web/src/surfaces/media/mediaSurface.test.tsx`
  - `apps/admin-web/src/surfaces/media/MediaWatchPage.tsx`
- Existing helper candidates:
  - `multiplePathTicket`
  - `setMediaTiming`
  - `createFixtureMediaDataSource`
