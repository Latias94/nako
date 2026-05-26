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
cd apps/media-web && npm run check && npm run test && npm run build
```

This gate becomes required after `apps/media-web` exists.

### Boundary Leakage Gate

```bash
rg -n "admin/v1|AdminApi|adminApi|source_locator|local path|ffmpeg" apps/media-web
```

This gate becomes required after `apps/media-web` exists. Any match must be
reviewed and either removed or justified as fixture/test-only safe text.

### Browser Smoke Gate

Use the local dev server and verify desktop and mobile viewports for:

- connect/login MVP;
- `/libraries`;
- `/libraries/:libraryId`;
- `/search`;
- `/items/:itemId`;
- `/watch/:itemId`.

The first smoke may use deterministic fixtures. Live smoke should be added when
a local server with test media is available.

## Evidence Log

| Date | Task | Evidence | Result |
| --- | --- | --- | --- |
| 2026-05-26 | MWF-010 split | `DESIGN.md`, `TODO.md`, `MILESTONES.md`, `EVIDENCE_AND_GATES.md`, `WORKSTREAM.json`, `HANDOFF.md` | Media Web foundation lane opened from client-surface planning. |

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

