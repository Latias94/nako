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

