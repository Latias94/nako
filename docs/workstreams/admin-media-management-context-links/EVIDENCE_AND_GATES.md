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

## Redaction Checks

Management Context Link UI and tests must reject or avoid rendering:

- bearer tokens;
- raw local filesystem paths;
- raw Source Locators;
- provider payloads or raw external URLs containing credentials;
- FFmpeg paths, argv, stderr, output paths, and staging paths;
- storage credentials, bucket paths, and cache handles;
- addon secrets, webhook secrets, or hosted addon HTML.
