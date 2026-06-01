# Generated Artifact Apply Repair Actions — Milestones

Status: Active
Last updated: 2026-06-02

## M0 — Lane Opening

Exit criteria:

- GAOR and WAGR closeouts are linked;
- repair action is explicitly not a blind retry;
- workstream state agrees across `TODO.md`, `TASKS.jsonl`, and
  `WORKSTREAM.json`.

Primary evidence:

- `DESIGN.md`
- `CONTEXT.jsonl`

## M1 — Repair Seam Proof

Exit criteria:

- current single/bulk apply execution seams are audited;
- stale-target rejection and idempotent replay gates are fresh;
- the lane records whether repair needs no backend mutation, a narrow wrapper,
  or Web-only recovery-context UX.

Primary gates:

- focused server `nextest`;
- `git diff --check`;
- `EVIDENCE_AND_GATES.md` decision note.

## M2 — Bounded Repair Action

Exit criteria:

- chosen repair contract is implemented;
- no second metadata apply executor exists;
- idempotency, freshness, authorization, redaction, and audit are covered by
  tests.

## M3 — Web Confirmation UX

Exit criteria:

- recovery queue exposes the repair preparation/confirmation path;
- fixture mode cannot claim live mutation;
- route/data-source tests and browser smoke pass.

## M4 — Closeout

Exit criteria:

- fresh evidence is recorded;
- architecture maps and workstream indexes reflect shipped repair behavior;
- provider-depth precision and broader diagnostics remain split as follow-ons.
