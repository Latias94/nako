# Web Admin Acquisition Intake - Evidence And Gates

Status: Active
Last updated: 2026-05-28

## Gate Set

Opening gate:

```bash
python -m json.tool docs/workstreams/web-admin-acquisition-intake/WORKSTREAM.json
git diff --check -- docs/workstreams/web-admin-acquisition-intake
```

Implementation gates:

```bash
npm --prefix web run test -- src/test/data-source-contracts.test.ts
npm --prefix web run test -- src/test/route-contracts.test.tsx src/test/route-state-contracts.test.tsx
npm --prefix web run check
npm --prefix web run build:budget
```

Closeout should also record browser smoke for desktop and mobile viewports.

## Evidence Log

| Date | Task | Evidence | Result |
| --- | --- | --- | --- |
| 2026-05-28 | WAAI-010 | Opened this lane from WDRP-030 after confirming the generated Admin contract exposes acquisition intake candidates and the old Admin Web V2 route is closed as prior art. | Passed. |
