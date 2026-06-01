# Web Admin Generated Artifact Recovery UI — Milestones

Status: Active
Last updated: 2026-06-02

## M0 — Lane Opening

Exit criteria:

- GAOR closeout is linked;
- the route is explicitly read-only;
- workstream state agrees across `TODO.md`, `TASKS.jsonl`, and
  `WORKSTREAM.json`.

Primary evidence:

- `DESIGN.md`
- `CONTEXT.jsonl`

## M1 — Recovery Route

Exit criteria:

- Web Admin has a recovery route reachable through route contracts;
- attention filtering, pagination, and summary counters render correctly;
- fixture and live read-model mapping are covered;
- responsive browser smoke has no overflow or raw internal leakage.

Primary gates:

- Web route and data-source tests;
- TypeScript check;
- browser smoke.

## M2 — Closeout

Exit criteria:

- fresh evidence is recorded;
- architecture maps and workstream indexes reflect the shipped route;
- mutation and deeper diagnostics remain split as follow-ons.
