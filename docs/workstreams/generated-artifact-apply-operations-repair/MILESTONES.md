# Generated Artifact Apply Operations Repair — Milestones

Status: Active
Last updated: 2026-06-02

## M0 — Scope And Evidence Freeze

Exit criteria:

- repair problem and non-goals are explicit;
- architecture routing and related closeouts are linked;
- first repair-oriented proof target is chosen;
- machine-readable workstream state agrees with the human ledger.

Primary evidence:

- `docs/workstreams/generated-artifact-apply-operations-repair/DESIGN.md`
- `docs/workstreams/generated-artifact-apply-operations-repair/TODO.md`

## M1 — Repair Read-Path Audit

Exit criteria:

- current durable outcome and batch records are mapped to operator needs;
- replay-vs-repair semantics are explicit;
- the smallest redaction-safe Admin repair read path is chosen;
- mutation is split if the audit proves read and repair execution should not
  land together.

Primary gates:

- focused Rust/API/Web contract gates selected by `GAOR-020`

## M2 — First Repair Surface

Exit criteria:

- operators can inspect repair-relevant Generated Artifact apply state through a
  safe Admin surface;
- if a repair action is added, it is confirmation-backed, idempotent, and
  reuses existing Metadata Authority apply semantics;
- Web/API copy and tests make stale/failed/noop/skipped outcomes understandable
  without raw internal leakage.

Primary gates:

- focused Rust/Web gates selected by `GAOR-020`
- browser smoke if a new Admin route or action is added

## M3 — Closeout

Exit criteria:

- gate set is recorded with fresh evidence;
- architecture maps and workstream indexes reflect the shipped repair surface;
- remaining larger operations automation or provider-depth work is either
  deferred or split into an explicit follow-on.
