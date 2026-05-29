# Web Playlist Management UI Mutations - Evidence And Gates

Status: Active
Last updated: 2026-05-29

## Gate Set

Opening gate:

```bash
python -m json.tool docs/workstreams/web-playlist-management-ui-mutations/WORKSTREAM.json
git diff --check -- docs/workstreams/web-playlist-management-ui-mutations
```

Frontend gates:

```bash
npm --prefix web run test -- src/test/data-source-contracts.test.ts
npm --prefix web run test -- src/test/route-contracts.test.tsx src/test/route-state-contracts.test.tsx
npm --prefix web run test
npm --prefix web run check
npm --prefix web run build:budget
```

SDK/API gates only if implementation discovers a route or generated SDK defect:

```bash
npm run check --prefix sdk/typescript
cargo nextest run -p nako-api playlist --no-fail-fast
cargo nextest run -p nako-server user_playlist --no-fail-fast
cargo nextest run -p nako-client user_playlist --no-fail-fast
```

Browser smoke should cover desktop and mobile viewports for create, rename,
delete, add, remove, reorder, and conflict/error states as each slice lands.

## Evidence Log

| Date | Task | Evidence | Result |
| --- | --- | --- | --- |
| 2026-05-29 | WPMU-010 | Opened this lane from the closed `user-playlists-contract-and-web-slice` follow-on. Scope is web playlist management mutations through Public Client only; sharing, smart lists, recommendation-generated lists, offline sync, and mobile/Tauri playlist surfaces remain separate lanes. Validation: `python -m json.tool docs/workstreams/web-playlist-management-ui-mutations/WORKSTREAM.json`; `git diff --check -- docs/workstreams/web-playlist-management-ui-mutations`. | Passed. |
| 2026-05-29 | WPMU-020 | Added Public Client-backed playlist mutation data-source methods for create, rename, delete, add item, remove item, and reorder; added TanStack Query mutation hooks with list/items cache invalidation and delete cache removal. Fixture mutation payloads report `persisted: false` and do not claim write success. Validation: `npm --prefix web run test -- src/test/data-source-contracts.test.ts src/test/use-media-contracts.test.tsx`; `npm --prefix web run check`. | Passed. |
| 2026-05-29 | WPMU-030 | Added `/media/my-list` create, rename, and delete controls backed by Public Client mutation hooks. Create selects the returned playlist, rename submits `expected_version`, delete moves the route to the next available playlist, and fixture mutation failures remain visible without changing URL state. Screenshots: `screenshots/wpmu-030-desktop.png`, `screenshots/wpmu-030-mobile.png`. Validation: `npm --prefix web run test -- src/test/route-contracts.test.tsx src/test/route-state-contracts.test.tsx`; `npm --prefix web run test`; `npm --prefix web run check`; Edge headless desktop/mobile screenshots. | Passed. |
| 2026-05-29 | WPMU-040 | Added playlist item removal from `/media/my-list` list rows and poster cards. Added a shared add-to-playlist dropdown used by media detail hero actions and the Recently Added browse cards. Route tests cover remove, detail add, and browse add through Public Client paths; data-source tests cover fixture add non-persistence. Browser smoke covered `/media` and `/media/detail` add menus plus visible fixture non-persistence feedback. Validation: `npm --prefix web run test -- src/test/route-state-contracts.test.tsx`; `npm --prefix web run test -- src/test/data-source-contracts.test.ts`; `npm --prefix web run test`; `npm --prefix web run check`. | Passed. |
| 2026-05-29 | WPMU-050 | Added explicit up/down reorder controls for playlist list rows and poster cards. Reorder submits full ordered `item_ids` with `expected_version`, keeps the current route state, and refetches the current item list when the mutation returns a stale-version conflict. Browser smoke covered list-mode enabled/disabled reorder controls and fixture non-persistence feedback. Validation: `npm --prefix web run test -- src/test/route-contracts.test.tsx src/test/route-state-contracts.test.tsx`; `npm --prefix web run test`; `npm --prefix web run check`; `python -m json.tool docs/workstreams/web-playlist-management-ui-mutations/WORKSTREAM.json`; `git diff --check -- web docs/workstreams/web-playlist-management-ui-mutations`. | Passed. |
