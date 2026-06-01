# Web MVP Live Smoke - Handoff

Status: Active
Last updated: 2026-06-01

## Current State

The workstream is opened under the `web-product` lane. WMLS-020 added the
dedicated MVP smoke artifact and WMLS-030 passed the required Web gate set.

## Next Task

Return to planner integration for WMLS-040. The planner should decide whether
this workstream can close as MVP Gate 3 evidence or whether a separate manual
browser screenshot/runbook is still required.

Fresh evidence recorded:

```bash
npm --prefix web run test -- src/test/mvp-live-smoke.test.tsx
npm --prefix web run test
npm --prefix web run check
npm --prefix web run build:budget
python -m json.tool docs/workstreams/web-mvp-live-smoke/WORKSTREAM.json
git diff --check -- docs/workstreams/web-mvp-live-smoke web docs/architecture/LANES.md docs/architecture/WORKSTREAM_LINKS.md docs/workstreams/README.md
```

## Stop Conditions

- Public Client or Admin API contract changes are needed.
- Backend route, generated SDK, schema, or Rust changes are needed.
- Desktop/Tauri native playback strategy enters scope.
- The smoke requires GAMA, CSAPA, or MVP planner task-ledger edits beyond
  linking this workstream.

## Integration Note

Return to the planner with `integrate-lane-results`. This lane should not start
desktop/native playback, backend/API, generated client, or MVP planner work.
