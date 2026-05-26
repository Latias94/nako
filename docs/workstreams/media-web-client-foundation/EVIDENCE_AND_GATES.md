# Media Web Client Foundation - Evidence And Gates

Status: Active
Last updated: 2026-05-26

## Smallest Current Repro

```bash
python -m json.tool docs/workstreams/media-web-client-foundation/WORKSTREAM.json
git diff --check -- docs/workstreams/media-web-client-foundation docs/workstreams/client-surface-and-access-product-architecture docs/workstreams/README.md
```

## Gate Set

### Planning Gate

```bash
python -m json.tool docs/workstreams/media-web-client-foundation/WORKSTREAM.json
git diff --check -- docs/workstreams/media-web-client-foundation docs/workstreams/client-surface-and-access-product-architecture docs/workstreams/README.md
```

This proves the workstream split is syntactically valid and does not introduce
whitespace errors.

### Public Client Contract Gate

```bash
cargo test -p nako-api public_openapi -- --nocapture
cargo run -q -p nako-api --example emit-typescript-sdk -- --output sdk/typescript/src/index.ts
git diff --check
```

This proves Media Web can rely on the Public Client API and generated TypeScript
SDK without using Admin API state.

### Media Web Package Gate

```bash
cd apps/admin-web && npm run check && npm run test && npm run build
```

This gate is required because Media and Admin now coexist inside the shared
frontend package.

### Boundary Leakage Gate

```bash
rg -n "admin/v1|AdminApi|adminApi|source_locator|local path|ffmpeg" apps/admin-web/src/surfaces/media
```

Any match must be reviewed and either removed or justified as fixture/test-only
safe text.

### Browser Smoke Gate

Use the local dev server and verify desktop and mobile viewports for:

- connect/login MVP;
- `/media/libraries`;
- `/media/libraries/:libraryId`;
- `/media/search`;
- `/media/items/:itemId`;
- `/media/watch/:itemId`.

The first smoke may use deterministic fixtures. Live smoke should be added when
a local server with test media is available.

## Evidence Log

| Date | Task | Evidence | Result |
| --- | --- | --- | --- |
| 2026-05-26 | MWF-010 split | `DESIGN.md`, `TODO.md`, `MILESTONES.md`, `EVIDENCE_AND_GATES.md`, `WORKSTREAM.json`, `HANDOFF.md` | Media Web foundation lane opened from client-surface planning. |
| 2026-05-26 | MWF-020 Public Client readiness | `cargo test -p nako-api public_openapi -- --nocapture`; `cargo run -q -p nako-api --example emit-typescript-sdk -- --output sdk/typescript/src/index.ts`; `ROUTE_API_READINESS.md` | DONE_WITH_CONCERNS. OpenAPI tests passed and SDK regeneration produced no content diff. First route matrix is recorded; app scaffold can proceed only inside the accepted boundary until public gaps are resolved. |
| 2026-05-26 | MWF-030 shared Media surface | `cd apps/admin-web && npm run test -- App.test.tsx mediaSurface.test.tsx mediaDataSource.test.ts`; `cd apps/admin-web && npm run check`; `cd apps/admin-web && npm run build`; `cd apps/admin-web && npm run test`; `rg -n "admin/v1\|AdminApi\|adminApi\|source_locator\|local path\|ffmpeg" apps/admin-web/src/surfaces/media`; `git diff --check`; Playwright smoke for `http://127.0.0.1:5174/media` and `/overview` | DONE. Focused tests passed 99/99; package tests passed 149/149; TypeScript check and Vite build passed; boundary grep returned no matches; browser smoke confirmed Media connect, fixture home, and Admin surface switch. `git diff --check` had no whitespace errors and only Windows LF-to-CRLF warnings. |

## Redaction And Safety Checks

Media Web must not expose these in normal viewer routes:

- bearer tokens after entry;
- password hashes, reset tokens, or invitation secrets;
- raw Source Locators;
- local filesystem paths;
- provider payloads;
- addon tokens or webhook secrets;
- storage credentials;
- FFmpeg argv, output paths, or raw stderr;
- Admin API policy rows, Role assignment internals, or policy reasons.

## Notes

Fresh verification is required before marking any task, goal, or lane complete.
