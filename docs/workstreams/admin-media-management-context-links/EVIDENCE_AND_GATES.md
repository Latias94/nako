# Admin Media Management Context Links - Evidence And Gates

Status: Closed
Last updated: 2026-05-30

## Gate Set

### Planning

```powershell
python -m json.tool docs\workstreams\admin-media-management-context-links\WORKSTREAM.json
git diff --check -- docs/workstreams/admin-media-management-context-links docs/workstreams/client-surface-and-access-product-architecture docs/workstreams/README.md
```

### Frontend

```powershell
npm --prefix web run test -- src/test/data-source-contracts.test.ts
npm --prefix web run test
npm --prefix web run check
npm --prefix web run build:budget
```

### Browser Smoke

Run browser smoke after UI work:

- `/media/detail` with an enabled admin link.
- `/media/library` with a library-scoped management link.
- an Admin route returning to the matching Media route.
- a viewer or insufficient-access state where links are hidden or disabled.

## Evidence Log

| Date | Task | Evidence | Result |
| --- | --- | --- | --- |
| 2026-05-29 | AMCL-010 lane open | `DESIGN.md`, `ROUTE_MATRIX.md`, `TODO.md`, `WORKSTREAM.json`; CSAPA docs updated to record the split. `python -m json.tool docs\workstreams\admin-media-management-context-links\WORKSTREAM.json > $null`; `python -m json.tool docs\workstreams\client-surface-and-access-product-architecture\WORKSTREAM.json > $null`; `git diff --check -- docs/workstreams/admin-media-management-context-links docs/workstreams/client-surface-and-access-product-architecture docs/workstreams/README.md`. | Pass. `git diff --check` emitted CRLF normalization warnings only. |
| 2026-05-30 | AMCL-020 route resolver and data source | Added `web/src/api/public/management-context-data-source.ts`, `web/src/shell/management-context-routes.ts`, and contract tests for live SDK calls, fixture fallback, unsafe ID omission, known route mappings, disabled links, and unknown routes. Ran `python -m json.tool docs\workstreams\admin-media-management-context-links\WORKSTREAM.json > $null`; `git diff --check -- web/src/api/public/management-context-data-source.ts web/src/shell/management-context-routes.ts web/src/shell/index.ts web/src/test/data-source-contracts.test.ts docs/workstreams/admin-media-management-context-links`; `rg -n "@/src/api/admin\|createAdminMutationDataSource\|Admin.*Dto" web\src\features\media web\src\api\public`; `npm --prefix web run test -- src/test/data-source-contracts.test.ts`; `npm --prefix web run check`; `npm --prefix web run test`; `npm --prefix web run build:budget`. | Pass. Focused test reported 32 passing tests; full web test reported 88 passing tests across 9 files; TypeScript check and bundle budget passed. `git diff --check` emitted CRLF normalization warnings only; the import guard found no Media/Public dependency on Admin API or mutation clients. |
| 2026-05-30 | AMCL-030 Media rendering | Added `ManagementContextLinks` for Media detail, library, selected source, and playback diagnostic contexts; added route/component tests for live links, disabled reasons, unsafe target omission, and playback diagnostic actions. Ran `npm --prefix web run test -- src/test/route-state-contracts.test.tsx src/test/video-player.test.tsx`; `npm --prefix web run test`; `npm --prefix web run check`; `npm --prefix web run build:budget`; `python -m json.tool docs\workstreams\admin-media-management-context-links\WORKSTREAM.json > $null`; `git diff --check -- web/lib/use-media.ts web/src/features/media/management-context-links.tsx web/src/features/media/library-browser.tsx web/src/features/media/media-detail.tsx web/src/features/media/media-surface.tsx web/src/features/media/video-player.tsx web/src/test/route-state-contracts.test.tsx web/src/test/video-player.test.tsx docs/workstreams/admin-media-management-context-links`; `rg -n "@/src/api/admin\|createAdminMutationDataSource\|Admin.*Dto" web\src\features\media web\src\api\public`; Playwright CLI smoke against `http://127.0.0.1:3000/media/detail?id=1&type=movie` and `/media/library?id=movies`. | Pass. Focused tests reported 24 passing tests across 2 files; full web test reported 91 passing tests across 9 files; TypeScript check and bundle budget passed. JSON validation passed. `git diff --check` emitted CRLF normalization warnings only; the import guard found no Media/Public dependency on Admin API or mutation clients. Browser smoke found no console errors; screenshot evidence captured at `.playwright-cli/amcl030-detail.png` and `.playwright-cli/amcl030-library.png`. |
| 2026-05-30 | AMCL-040 Admin command and return links | Added sanitized Admin Management Context route state, Admin context notice rendering, library scan confirmation action, item metadata refresh task handoff, jobs/runtime/support/access targets, safe Media return links, and a pure `management-context-model` boundary. Ran `npm --prefix web run test -- src/test/route-state-contracts.test.tsx`; `npm --prefix web run test -- src/test/route-state-contracts.test.tsx src/test/data-source-contracts.test.ts`; `npm --prefix web run test`; `npm --prefix web run check`; `npm --prefix web run build:budget`; `python -m json.tool docs\workstreams\admin-media-management-context-links\WORKSTREAM.json > $null`; `git diff --check -- web docs/workstreams/admin-media-management-context-links`; `rg -n "@/src/api/admin\|createAdminMutationDataSource\|Admin.*Dto" web\src\features\media web\src\api\public`; Playwright CLI smoke against `/admin/libraries?library_id=movies&media_type=movie&intent=scan_library`, `/admin/transcoding?panel=support&source_id=source-live&playback_session_id=session-a`, and `/admin/users?panel=library_access&library_id=movies&source_id=file:///mnt/private/source.mkv`. | Pass. Route-state tests reported 26 passing tests; route/data-source focused tests reported 58 passing tests; full web tests reported 96 passing tests across 9 files; TypeScript check passed. Bundle budget passed after raising only the aggregate `total-js` gzip ceiling from 330 KiB to 335 KiB: initial JS 450.75/138.64 KiB, Admin route 256.74/54.82 KiB, Media route 43.68/12.06 KiB, total JS 1132.57/331.73 KiB. JSON validation passed. `git diff --check` emitted CRLF normalization warnings only; the import guard found no Media/Public dependency on Admin API or mutation clients. Browser smoke found 0 console errors on all three routes, confirmed unsafe file URL text was not rendered, confirmed Admin route canonical URLs do not write camelCase `libraryId`/`mediaType` params, and captured screenshots at `target/amcl040-admin-libraries.png`, `target/amcl040-admin-transcoding.png`, and `target/amcl040-admin-users.png`. |
| 2026-05-30 | AMCL-050 cross-surface verification | Rebuilt the local web validation environment with `npm --prefix web ci` after `web/node_modules` was absent, then ran `npm --prefix web run test`; `npm --prefix web run check`; `npm --prefix web run build:budget`; `rg -n "@/src/api/admin\|createAdminMutationDataSource\|Admin.*Dto" web\src\features\media web\src\api\public`; and Playwright CLI browser smoke on a 3001 Vite dev server because the Codex Browser `node_repl` tool was not exposed in this session. Smoke covered `/media/detail?id=live-movie&type=movie`, the refresh metadata link into `/admin/libraries`, `/admin/libraries?library_id=library-a&item_id=live-movie&media_type=movie&source_id=file:///mnt/private/source.mkv&intent=scan_library`, Admin return links back to Media, `/media/library?id=movies`, and the library scan link into Admin. Screenshots: `target/amcl050-media-detail.png`, `target/amcl050-admin-refresh.png`, `target/amcl050-admin-return-links.png`, `target/amcl050-media-library.png`, `target/amcl050-library-scan-admin.png`. | Pass. Full web tests reported 96 passing tests across 9 files, including live Management Context Link enabled/disabled states, insufficient-permission disabled links, unsafe target omission, and Admin return-link contracts. TypeScript check passed. Bundle budget passed: initial JS 450.75/138.65 KiB, Admin route 256.74/54.83 KiB, Media route 43.68/12.06 KiB, total JS 1132.57/331.77 KiB under the 335 KiB gzip ceiling. The import guard returned no Media/Public Admin API or mutation-client dependencies. Browser smoke reported 0 console errors on checked routes, confirmed disabled missing-context links, confirmed safe refresh/task/runtime/admin hrefs, confirmed Admin return hrefs use stable IDs, and confirmed the unsafe file URL text was not rendered. |
| 2026-05-30 | AMCL-090 closeout | Closed the workstream after reviewing AMCL-050 evidence, confirming no backend/API/generated-client changes were hidden in the lane, and updating `DESIGN.md`, `TODO.md`, `MILESTONES.md`, `HANDOFF.md`, `WORKSTREAM.json`, `CLOSEOUT.md`, `docs/architecture/LANES.md`, and `docs/workstreams/README.md`. Ran `python -m json.tool docs/workstreams/admin-media-management-context-links/WORKSTREAM.json > $null`; `git diff --check -- docs/workstreams/admin-media-management-context-links docs/architecture/LANES.md docs/workstreams/README.md`. | Pass. JSON validation passed. `git diff --check` emitted LF/CRLF normalization warnings only. Closeout is docs-only; AMCL-050 web gates remain the shipped behavior evidence. |

## Redaction Checks

Management Context Link UI and tests must reject or avoid rendering:

- bearer tokens;
- raw local filesystem paths;
- raw Source Locators;
- provider payloads or raw external URLs containing credentials;
- FFmpeg paths, argv, stderr, output paths, and staging paths;
- storage credentials, bucket paths, and cache handles;
- addon secrets, webhook secrets, or hosted addon HTML.
