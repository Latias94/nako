# Web Admin Generated Artifact Recovery UI — TODO

Status: Closed
Last updated: 2026-06-02

## M0 — Lane Opening

- [x] WAGR-010 [owner=planner] [deps=none] [scope=docs/workstreams/web-admin-generated-artifact-recovery-ui,docs/architecture,docs/workstreams/README.md,docs/GOALS.md,docs/ROADMAP.md]
  Goal: Open the Web Admin Generated Artifact recovery UI lane from GAOR closeout evidence and freeze the read-only product boundary.
  Validation: `python -m json.tool docs/workstreams/web-admin-generated-artifact-recovery-ui/WORKSTREAM.json`; JSONL validation for `TASKS.jsonl` and `CAMPAIGNS.jsonl`; `git diff --check`.
  Evidence: `DESIGN.md`, `WORKSTREAM.json`, `CONTEXT.jsonl`.
  Handoff: Execution begins at `WAGR-020`.

## M1 — Recovery Route

- [x] WAGR-020 [owner=codex] [deps=WAGR-010] [scope=web/src/features/admin,web/src/shell,web/src/api/admin,web/src/test]
  Goal: Add a read-only Web Admin route for Generated Artifact apply recovery with attention filters, summary counters, pagination, and redaction-safe row facts.
  Validation: `npm --prefix web run test -- src/test/data-source-contracts.test.ts`; `npm --prefix web run test -- src/test/route-contracts.test.tsx src/test/route-state-contracts.test.tsx`; `npm --prefix web run check`; browser smoke when route renders.
  Review: confirm no mutation controls, no raw prompt/path/provider leakage, and no responsive overflow.
  Evidence: `EVIDENCE_AND_GATES.md`.
  Handoff: DONE. Continued to `WAGR-030` closeout; no backend read-model gap was found.

## M2 — Closeout

- [x] WAGR-030 [owner=planner] [deps=WAGR-020] [scope=docs/workstreams/web-admin-generated-artifact-recovery-ui,docs/architecture]
  Goal: Close the lane after route smoke, or split a narrow follow-on if repair mutation or deeper diagnostics become necessary.
  Validation: fresh gate evidence in `EVIDENCE_AND_GATES.md`; `git diff --check`.
  Review: review-workstream has no blocking findings.
  Evidence: `CLOSEOUT.md`.
  Handoff: DONE. Repair mutation remains split to `proposed:generated-artifact-apply-repair-actions`.
