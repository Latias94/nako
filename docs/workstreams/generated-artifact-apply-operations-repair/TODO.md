# Generated Artifact Apply Operations Repair — TODO

Status: Active
Last updated: 2026-06-02

## M0 — Scope And Evidence Freeze

- [x] GAOR-010 [owner=planner] [deps=none] [scope=docs/workstreams/generated-artifact-apply-operations-repair,docs/architecture/LANES.md,docs/architecture/WORKSTREAM_LINKS.md,docs/architecture/LIBRARY_PIPELINE.md,docs/architecture/CONTROL_PLANE.md,docs/workstreams/README.md]
  Goal: Open the Generated Artifact apply operations repair lane from GAMA/GABMA/GAPM closeout evidence and freeze the operator-facing recovery problem, non-goals, and first proof boundary.
  Validation: `python -m json.tool docs/workstreams/generated-artifact-apply-operations-repair/WORKSTREAM.json`; JSONL validation for `TASKS.jsonl` and `CAMPAIGNS.jsonl`; `git diff --check -- docs/workstreams/generated-artifact-apply-operations-repair docs/architecture/LANES.md docs/architecture/WORKSTREAM_LINKS.md docs/architecture/LIBRARY_PIPELINE.md docs/architecture/CONTROL_PLANE.md docs/workstreams/README.md`.
  Evidence: `DESIGN.md`, `MILESTONES.md`, `CONTEXT.jsonl`.
  Handoff: Execution begins at `GAOR-020`.

## M1 — Repair Read-Path Audit

- [x] GAOR-020 [owner=codex] [deps=GAOR-010] [scope=crates/nako-core,crates/nako-api,crates/nako-server/src/app/automation.rs,crates/nako-server/src/http/admin.rs,web/src/api/admin]
  Goal: Audit current one-artifact outcome and bulk batch persistence/DTO/read-model coverage, then define the smallest redaction-safe Admin repair read path for stale/noop/failed/skipped apply outcomes.
  Validation: focused Rust/API/Web contract gates for the chosen read path; `cargo fmt --all -- --check`; `npm --prefix web run check` if Web/Admin contract shape changes.
  Review: confirm replay-vs-repair semantics, target freshness visibility, and no raw prompt/path/provider leakage.
  Evidence: `EVIDENCE_AND_GATES.md` plus route/API/read-model notes in this workstream.
  Handoff: Split mutation follow-on if the audit proves read-path work and repair execution should not land together.

## M2 — First Repair Surface

- [x] GAOR-030 [owner=codex] [deps=GAOR-020] [scope=crates/nako-core,crates/nako-db,crates/nako-api,crates/nako-server,web/src/api/admin]
  Goal: Ship the first Admin-facing repair surface, either as a read-only recovery queue/detail route or as a bounded repair action that reuses existing apply semantics.
  Validation: focused Rust/Web gates chosen by `GAOR-020`; browser smoke if a Web route is added; `npm --prefix web run build:budget` when route code changes.
  Review: review-workstream before accepting completion.
  Evidence: `EVIDENCE_AND_GATES.md`.
  Handoff: Continue to `GAOR-040` for closeout or split a narrower follow-on if execution scope grows.

## M3 — Closeout

- [ ] GAOR-040 [owner=planner] [deps=GAOR-030] [scope=docs/workstreams/generated-artifact-apply-operations-repair,docs/architecture]
  Goal: Close the lane or split a narrower follow-on for broader operations automation, provider precision, or cross-lane diagnostics.
  Validation: verify-rust-workstream or equivalent fresh gate evidence recorded in `EVIDENCE_AND_GATES.md`; `git diff --check`.
  Review: review-workstream has no blocking findings.
  Evidence: `CLOSEOUT.md` if closed, otherwise updated `HANDOFF.md` and split references.
  Handoff: DONE or explicit follow-on split only.
