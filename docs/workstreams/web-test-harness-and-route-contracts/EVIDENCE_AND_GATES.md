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
| 2026-05-28 | WTRC-020 | `npm --prefix web install -D vitest @testing-library/react @testing-library/jest-dom @testing-library/user-event jsdom`; added `web/vitest.config.ts`, `web/src/test/setup.ts`, and `web/src/test/harness.test.ts`; `npm --prefix web run test`; `npm --prefix web run check`. | Passed. `npm test` now runs Vitest instead of aliasing type-check. |

## Notes

Do not count `npm run check` as test coverage after WTRC-020. Type-check remains
a separate gate.
