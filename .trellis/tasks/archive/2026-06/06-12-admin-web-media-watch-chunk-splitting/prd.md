# Admin Web Media Watch Chunk Splitting

## Goal

Reduce Admin Web media route bundle coupling by moving watch/playback-only code
out of the broad `MediaPages` route chunk.

## Background

The previous route-level bundle split moved Admin Web pages behind lazy route
imports, but `apps/admin-web/src/surfaces/media/MediaPages.tsx` still contains
browse/search/library pages together with the watch page and browser playback
logic. That keeps watch/playback-only code in the same Vite chunk as lighter
media browsing routes.

## Requirements

- Split the media watch route implementation into a dedicated module loaded by
  the `/media/watch/$itemId` route.
- Keep `/media`, `/media/libraries`, `/media/search`, and `/media/items/$itemId`
  behavior unchanged.
- Keep route-owned search types and URL normalization in `App.tsx`.
- Preserve existing Media Web playback behavior, token redaction behavior, and
  route tests.
- Avoid broad product UX changes; this is a structural bundle-boundary task.

## Acceptance Criteria

- `App.tsx` lazy-loads the media watch route from the watch-specific module.
- TypeScript check and affected route tests pass.
- `npm run build --prefix apps/admin-web` emits a separate watch-related chunk,
  and the remaining `MediaPages-*` chunk is smaller or no longer contains
  watch/playback implementation code.
- The Admin Web route-level bundle splitting spec is updated if a durable
  convention is learned.

## Validation

- `npm run check --prefix apps/admin-web`
- `npm run test --prefix apps/admin-web -- surfaces/media/mediaSurface.test.tsx App.test.tsx`
- `npm run build --prefix apps/admin-web`
- `git diff --check`
