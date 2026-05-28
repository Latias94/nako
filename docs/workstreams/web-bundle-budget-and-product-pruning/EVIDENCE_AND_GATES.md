# Web Bundle Budget And Product Pruning - Evidence And Gates

Status: Active
Last updated: 2026-05-28

## Gate Set

```bash
npm --prefix web run test
npm --prefix web run check
npm --prefix web run build
npm --prefix web run tauri -- build
git diff --check
```

Bundle budget output must be recorded before and after pruning.

## Evidence Log

| Date | Task | Evidence | Result |
| --- | --- | --- | --- |
| 2026-05-28 | WBBP-010 | Queued as lane 6 after Admin live wiring. | Queued. |
| 2026-05-28 | WBBP-010 | WALW-050 closed at commit `ee6d5cdc`; WBBP status moved to active and current task set to WBBP-020. | Passed. |
| 2026-05-28 | WBBP-020 | Added `web/scripts/check-bundle-budget.mjs`, `bundle:budget`, and `build:budget`. `npm --prefix web run build:budget` passed with initial JS 442.74 KiB / 136.39 KiB gzip, initial CSS 195.92 / 28.34, admin route 212.59 / 45.67, media route 320.37 / 72.94, total JS 1148.54 / 312.15. | Passed. |
| 2026-05-28 | WBBP-030 | Lazy-loaded accepted media subviews, removed v0-only deferred media domains from the live runtime graph, and deleted their prototype files. `npm --prefix web run check`, `npm --prefix web run test`, and `npm --prefix web run build:budget` passed with media route 51.67 KiB / 14.38 KiB gzip and total JS 1041.80 KiB / 307.34 KiB gzip. | Passed. |
