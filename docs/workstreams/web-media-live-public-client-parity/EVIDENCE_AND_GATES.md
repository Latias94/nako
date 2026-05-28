# Web Media Live Public Client Parity - Evidence And Gates

Status: Active
Last updated: 2026-05-28

## Gate Set

Planning gate:

```bash
python -m json.tool docs/workstreams/web-media-live-public-client-parity/WORKSTREAM.json
git diff --check -- docs/workstreams/web-media-live-public-client-parity docs/workstreams/web-deferred-product-reentry-plan docs/workstreams/README.md
```

Implementation gates:

```bash
npm --prefix web run test
npm --prefix web run check
npm --prefix web run build:budget
npm --prefix web run tauri -- build
git diff --check
```

Browser smoke gates should cover:

- `/media`
- `/media/search?q=dune`
- `/media/detail?id=<live-or-fixture-id>&type=movie`
- `/media/library?id=<library-id>`
- playback entry path when WMLP-040 is active

## Evidence Log

| Date | Task | Evidence | Result |
| --- | --- | --- | --- |
| 2026-05-28 | WMLP-010 | Opened this lane from WDRP-020 after inspecting current `web/src/api/public` live seams, generated SDK playback methods, and Media Web foundation follow-ons. | Passed. |
| 2026-05-28 | WMLP-020 | Audited committed `sdk/typescript/src/index.ts`, committed Public Client route inventory, and current `web/src/api/public/media-data-source.ts`; updated `ROUTE_API_READINESS.md` with ready contracts and gaps. Ran `npm --prefix web run test -- src/test/data-source-contracts.test.ts`, `npm --prefix web run check`, `python -m json.tool docs/workstreams/web-media-live-public-client-parity/WORKSTREAM.json`, and `git diff --check -- docs/workstreams/web-media-live-public-client-parity`. | Passed: 1 Vitest file / 13 tests passed; TypeScript `tsc --noEmit` passed; JSON and scoped diff checks passed. |
