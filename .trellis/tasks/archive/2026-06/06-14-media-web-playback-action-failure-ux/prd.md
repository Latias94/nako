# Media Web Playback Action Failure UX

## Goal

Make the recently added Media Web playback-state actions resilient when the Public Client mutation fails, starting with the Home Continue Watching row-level Start over action.

## What I Already Know

- Continue Watching row-level Start over now calls `MediaWebDataSource.setUserWatchedState`.
- On success, fixture mode refreshes Continue Watching so the cleared row disappears.
- Current code stores a generic mutation error, but the failure path is not covered by tests.
- Existing item detail playback-state actions already expose mutation errors through `MediaPlaybackState`.
- Admin Web tests must assert data-source calls, visible failures, and redaction constraints.

## Requirements

- Keep backend, SDK, database, and Public Client API unchanged.
- If Continue Watching row-level Start over fails, keep the original row visible.
- Show a safe, bounded error message for the failed action.
- Re-enable the row action after failure so the user can retry.
- Clear the previous error before the next retry attempt.
- Preserve redaction rules: do not render ticket tokens, stream URLs, bearer tokens, fingerprints, raw paths, source internals, or raw backend error text.

## Acceptance Criteria

- [x] A rejected Continue Watching Start over mutation leaves the row visible.
- [x] The Start over button is usable again after failure.
- [x] The visible error is safe and does not include raw backend/transport details.
- [x] A subsequent successful retry clears the row from Continue Watching.
- [x] Existing success-path Continue Watching tests still pass.
- [x] Admin Web check, focused Media Web tests, full Admin Web tests, and build pass.

## Definition of Done

- Tests added or updated for failure and retry behavior.
- `npm run test --prefix apps/admin-web -- src/surfaces/media/mediaSurface.test.tsx` passes.
- `npm run check --prefix apps/admin-web` passes.
- `npm run test --prefix apps/admin-web` passes.
- `npm run build --prefix apps/admin-web` passes.
- Trellis task validates and is archived after completion.

## Technical Approach

- Stay inside `apps/admin-web/src/surfaces/media/MediaPages.tsx` and `mediaSurface.test.tsx`.
- Reuse the existing `continueWatchingMutationError` state.
- Keep the current static safe error copy instead of rendering thrown error messages.
- Add a regression test that mocks `setUserWatchedState` to reject once, verifies the row and safe error remain, then retries successfully and verifies the row is removed.

## Decision (ADR-lite)

**Context**: Continue Watching now has a mutation action on the Home rail. A failed write must not make the UI look like progress was cleared.

**Decision**: Keep failure handling local to the Home Continue Watching action and render static safe copy rather than raw errors.

**Consequences**: The slice hardens the new mutation entrypoint without introducing a global mutation framework or expanding Public Client contracts.

## Out of Scope

- Backend, SDK, database, or Public Client API changes.
- Global notification/toast infrastructure.
- Retrying automatically.
- Changing watch-page browser-player Start over behavior.
- Redesigning item detail playback-state controls.

## Technical Notes

- Files inspected:
  - `apps/admin-web/src/surfaces/media/MediaPages.tsx`
  - `apps/admin-web/src/surfaces/media/MediaItemShared.tsx`
  - `apps/admin-web/src/surfaces/media/mediaSurface.test.tsx`
  - `.trellis/spec/admin-web/frontend/routes-forms-and-tests.md`
