# Notes

## 2026-06-14

- Scope stayed frontend-only in `apps/admin-web`.
- Continue Watching row-level Start over reuses `MediaWebDataSource.setUserWatchedState`.
- Payload uses `duration_ms` and saved `source_id` from the Continue Watching entry, plus `position_ms: 0` and `watched: false`.
- Fixture mode refreshes the Continue Watching query after mutation so the cleared item disappears from the Home rail.

## Verification

- `npm run test --prefix apps/admin-web -- src/surfaces/media/mediaSurface.test.tsx` passed.
- `npm run check --prefix apps/admin-web` passed.
- `npm run test --prefix apps/admin-web` passed.
- `npm run build --prefix apps/admin-web` passed.
- `git diff --check` passed.
- `trellis-check` sub-agent found no issues.

## Spec Update Review

- No `.trellis/spec` update needed: this follows existing Admin Web route/data-source/test conventions and does not add a new API, storage contract, infra integration, or reusable project convention.
