# Web Test Harness And Route Contracts - Evidence And Gates

Status: Active
Last updated: 2026-05-28

## Gate Set

```bash
npm --prefix web run test
npm --prefix web run check
npm --prefix web run build
git diff --check
```

Run static Playwright smoke when route rendering behavior changes visibly.

## Evidence Log

| Date | Task | Evidence | Result |
| --- | --- | --- | --- |
| 2026-05-28 | WTRC-010 | Workstream opened as the first lane in the six-lane frontend refactor roadmap. | Active. |

## Notes

Do not count `npm run check` as test coverage after WTRC-020. Type-check remains
a separate gate.
