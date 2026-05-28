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
