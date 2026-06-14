# Notes

## 2026-06-14

- Initial scope is frontend-only.
- Live mode stays read-after-write through the Public Client API.
- Fixture mode needs in-memory User Playback State to avoid fixed Continue Watching snapshots after watch-page mutations.
- Implemented fixture playback state as data-source-instance memory shared by `getUserPlaybackState()`, write mutations, and `listContinueWatching()`.
- Added data-source and route tests for progress refresh, source continuity, watched removal, and redaction.

## Verification

- `npm run test --prefix apps/admin-web -- src/surfaces/media/mediaDataSource.test.ts`
- `npm run test --prefix apps/admin-web -- src/surfaces/media/mediaSurface.test.tsx`
- `npm run check --prefix apps/admin-web`
- `npm run test --prefix apps/admin-web`
- `npm run build --prefix apps/admin-web`
- `git diff --check`
