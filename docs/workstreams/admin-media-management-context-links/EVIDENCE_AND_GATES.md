# Admin Media Management Context Links - Evidence And Gates

Status: Active
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

## Redaction Checks

Management Context Link UI and tests must reject or avoid rendering:

- bearer tokens;
- raw local filesystem paths;
- raw Source Locators;
- provider payloads or raw external URLs containing credentials;
- FFmpeg paths, argv, stderr, output paths, and staging paths;
- storage credentials, bucket paths, and cache handles;
- addon secrets, webhook secrets, or hosted addon HTML.
