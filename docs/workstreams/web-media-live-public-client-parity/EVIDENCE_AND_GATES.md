# Web Media Live Public Client Parity - Evidence And Gates

Status: Completed
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
| 2026-05-28 | WMLP-030A | Added Public Media read-model boundaries in `web/src/api/public/media-data-source.ts`: list readiness, detail sources/images, library summaries, library source readiness, and explicit missing library-scoped item browse state. Added data-source contract tests for detail and library readiness. Ran `npm --prefix web run test -- src/test/data-source-contracts.test.ts`, `npm --prefix web run check`, and `npm --prefix web run build:budget`. | Passed: 1 Vitest file / 16 tests passed; TypeScript `tsc --noEmit` passed; bundle budget passed with media route JS 56.81 KiB raw / 16.11 KiB gzip and total JS 1048.30 KiB raw / 309.49 KiB gzip. |
| 2026-05-28 | WMLP-030 | Routed Media browse/detail/search/library through Public Media read models where contracts exist. `/media/detail` now receives live item/source/image readiness; `/media/search` uses Public Client search; `/media/library` shows library metadata/source readiness and keeps scoped item browse as a missing contract. Ran `npm --prefix web run test`, `npm --prefix web run check`, and `npm --prefix web run build:budget`. | Passed: 8 Vitest files / 47 tests passed; TypeScript `tsc --noEmit` passed; bundle budget passed with media route JS 42.45 KiB raw / 11.62 KiB gzip and total JS 1051.05 KiB raw / 310.97 KiB gzip. |
| 2026-05-28 | WMLP-040 | Wired browser-ticket playback planning through Public Media data source and native `VideoPlayer` media/subtitle elements. Added no-token assertions for media and subtitle URLs. Verified that `BrowserPlaybackTicketResponse` does not expose a playback session id, so heartbeat is split as a Public Client contract follow-on. Ran `npm --prefix web run test`, `npm --prefix web run check`, and `npm --prefix web run build:budget`. | Passed: 8 Vitest files / 47 tests passed; TypeScript `tsc --noEmit` passed; bundle budget passed with media route JS 42.45 KiB raw / 11.62 KiB gzip and total JS 1051.05 KiB raw / 310.97 KiB gzip. |
| 2026-05-28 | WMLP-050 | Added Public Media playback-state read/write methods for continue-watching, progress, and watched state. Home continue-watching now reads Public Client playback-state data with fixture fallback. Ran `npm --prefix web run test`, `npm --prefix web run check`, and `npm --prefix web run build:budget`. | Passed: 8 Vitest files / 49 tests passed; TypeScript `tsc --noEmit` passed; bundle budget passed with media route JS 42.76 KiB raw / 11.75 KiB gzip and total JS 1053.15 KiB raw / 311.69 KiB gzip. |
| 2026-05-28 | WMLP-060 | Closed the lane with final validation: `npm --prefix web run test`, `npm --prefix web run check`, `npm --prefix web run build:budget`, `npm --prefix web run tauri -- build`, browser smoke for `/media`, `/media/search?q=dune`, `/media/detail?id=1&type=movie`, and `/media/library?id=movies`, `python -m json.tool docs/workstreams/web-media-live-public-client-parity/WORKSTREAM.json`, and `git diff --check`. | Passed: 8 Vitest files / 49 tests passed; TypeScript passed; bundle budget passed with media route JS 42.76 KiB raw / 11.75 KiB gzip and total JS 1053.15 KiB raw / 311.69 KiB gzip; Tauri built `nako-web-shell.exe`; browser smoke found no console errors. |
