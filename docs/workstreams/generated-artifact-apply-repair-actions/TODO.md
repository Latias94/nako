# Generated Artifact Apply Repair Actions — TODO

Status: Active
Last updated: 2026-06-02

## M0 — Lane Opening

- [x] GAARA-010 [owner=planner] [deps=none] [scope=docs/workstreams/generated-artifact-apply-repair-actions,docs/architecture,docs/workstreams/README.md,docs/GOALS.md,docs/ROADMAP.md]
  Goal: Open the bounded repair-actions lane from GAOR and WAGR closeout evidence.
  Validation: `python -m json.tool docs/workstreams/generated-artifact-apply-repair-actions/WORKSTREAM.json`; JSONL validation for `TASKS.jsonl` and `CAMPAIGNS.jsonl`; `git diff --check`.
  Evidence: `DESIGN.md`, `WORKSTREAM.json`, `CONTEXT.jsonl`.
  Handoff: Execution begins at `GAARA-020`.

## M1 — Repair Seam Proof

- [x] GAARA-020 [owner=codex] [deps=GAARA-010] [scope=crates/nako-core/src/automation.rs,crates/nako-api/src/admin/automation.rs,crates/nako-server/src/app/automation.rs,crates/nako-server/src/http/admin.rs,crates/nako-server/src/app/tests/automation.rs,crates/nako-server/src/http/tests,web/src/api/admin,web/src/features/admin,web/src/test,docs/workstreams/generated-artifact-apply-repair-actions]
  Goal: Prove the repair action seam and decide whether existing single/bulk apply routes are enough, a narrow recovery-context wrapper is required, or Web-only repair preparation is sufficient.
  Validation: `cargo nextest run -p nako-server generated_artifact_metadata_apply_replays_same_idempotency_key_from_durable_outcome generated_artifact_metadata_apply_rejects_stale_target_before_mutation --no-fail-fast`; add focused Rust/Web tests if new DTO or UX is introduced; `npm --prefix web run check` if Web changes; `git diff --check`.
  Review: no second metadata executor, no blind retry, no raw leakage, no mutation without fresh apply semantics.
  Evidence: `EVIDENCE_AND_GATES.md`; `web/src/test/route-state-contracts.test.tsx`.
  Handoff: Existing single/bulk apply routes are sufficient. Do not continue to `GAARA-030` automatically; move to `GAARA-050` closeout/split unless one-click recovery-row repair is approved as a new product requirement.

## M2 — Bounded Repair Action

- [ ] GAARA-030 [owner=codex] [deps=GAARA-020] [scope=crates/nako-core/src/automation.rs,crates/nako-api/src/admin/automation.rs,crates/nako-server/src/app/automation.rs,crates/nako-server/src/http/admin.rs,crates/nako-server/src/app/tests/automation.rs,crates/nako-server/src/http/tests,crates/nako-db/src,web/src/api/admin/generated,web/src/api/admin]
  Goal: Implement the chosen bounded repair action contract, reusing existing Metadata Authority apply or bulk apply execution.
  Validation: focused `cargo nextest run` for API/server/db contracts; `cargo check -p nako-server --tests`; regenerate/check Admin TypeScript contracts if DTOs change.
  Review: idempotent replay, stale-target rejection, redaction, authorization, and audit context are explicit.
  Evidence: `EVIDENCE_AND_GATES.md`.
  Handoff: Deferred by `GAARA-020`; run only if product chooses a one-click recovery-context wrapper with guards beyond the existing apply route.

## M3 — Web Confirmation UX

- [ ] GAARA-040 [owner=codex] [deps=GAARA-030] [scope=web/src/features/admin,web/src/api/admin,web/src/shell,web/src/test]
  Goal: Add Web Admin repair preparation and confirmation from the recovery queue without enabling fixture-mode mutation.
  Validation: `npm --prefix web run test -- src/test/data-source-contracts.test.ts`; route/state tests for recovery repair UX; `npm --prefix web run check`; `npm --prefix web run build:budget`; browser smoke.
  Review: no raw prompt/path/provider/token leakage, no confusing replay-only state as actionable repair, no responsive overflow.
  Evidence: `EVIDENCE_AND_GATES.md`.
  Handoff: Deferred by `GAARA-020`; current Web already prepares repair through the current apply plan and a new idempotency key. Run only for explicit UX copy/confirmation polish.

## M4 — Closeout

- [ ] GAARA-050 [owner=planner] [deps=GAARA-020] [scope=docs/workstreams/generated-artifact-apply-repair-actions,docs/architecture,docs/GOALS.md,docs/ROADMAP.md]
  Goal: Close the lane or split remaining repair diagnostics into a follow-on.
  Validation: fresh gate evidence in `EVIDENCE_AND_GATES.md`; JSON/JSONL validation; `git diff --check`.
  Review: review-workstream has no blocking findings.
  Evidence: `CLOSEOUT.md` if closed, otherwise updated `HANDOFF.md`.
  Handoff: DONE or explicit follow-on split only.
