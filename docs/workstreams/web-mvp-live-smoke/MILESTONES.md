# Web MVP Live Smoke - Milestones

Status: Active
Last updated: 2026-06-01

## M0 - Lane Opened

Exit criteria:

- `docs/workstreams/web-mvp-live-smoke/` exists with design, task ledger,
  context manifest, evidence gates, machine-readable workstream metadata, and
  handoff.
- The workstream is linked to the `web-product` lane.

## M1 - Smoke Artifact Landed

Exit criteria:

- A dedicated Web MVP smoke test exists under `web/src/test`.
- The smoke covers the route, Public Client data-source, browser-ticket,
  native `VideoPlayer`, heartbeat, and redaction expectations listed in MVP
  Gate 3.
- Targeted Vitest execution passes.

## M2 - Web Gate Evidence

Exit criteria:

- `npm --prefix web run test` passes.
- `npm --prefix web run check` passes.
- `npm --prefix web run build:budget` passes.
- `git diff --check -- docs/workstreams/web-mvp-live-smoke web` passes.
- Fresh evidence is recorded in `EVIDENCE_AND_GATES.md`.

## M3 - Planner Integration Ready

Exit criteria:

- The final worker report includes status, changed files, validation, evidence,
  concerns/follow-ups, and review readiness.
- The planner can route this through `integrate-lane-results`.
- Any remaining risk is outside this Web MVP smoke lane.

