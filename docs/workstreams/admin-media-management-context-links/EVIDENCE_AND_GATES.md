# Admin Media Management Context Links - Evidence And Gates

Status: Active
Last updated: 2026-05-29

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

## Redaction Checks

Management Context Link UI and tests must reject or avoid rendering:

- bearer tokens;
- raw local filesystem paths;
- raw Source Locators;
- provider payloads or raw external URLs containing credentials;
- FFmpeg paths, argv, stderr, output paths, and staging paths;
- storage credentials, bucket paths, and cache handles;
- addon secrets, webhook secrets, or hosted addon HTML.
