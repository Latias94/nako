# Provider Governance Bulk Review - TODO

Status: Active
Last updated: 2026-06-02

## M0 - Lane Opening

- [x] PGBR-010 [owner=planner] [deps=none] [scope=docs/workstreams/provider-governance-bulk-review,docs/architecture,docs/GOALS.md,docs/ROADMAP.md]
  Goal: Open the Provider Governance bulk review lane split from PRGQ closeout.
  Validation: `python -m json.tool docs/workstreams/provider-governance-bulk-review/WORKSTREAM.json`; JSONL validation for `TASKS.jsonl`, `CAMPAIGNS.jsonl`, and `CONTEXT.jsonl`; `git diff --check`.
  Review: opening must not add runtime behavior, schema migration, Public Client API, Web feature code, batch mutation, or hierarchy application.
  Evidence: `DESIGN.md`; `WORKSTREAM.json`; `EVIDENCE_AND_GATES.md`.
  Context: `CONTEXT.jsonl`.
  Handoff: DONE.

## M1 - Read-Only Batch Plan

- [x] PGBR-020 [owner=codex] [deps=PGBR-010] [scope=crates/nako-metadata,crates/nako-api,crates/nako-server,web/src/api/admin/generated,docs/workstreams/provider-governance-bulk-review]
  Goal: Add a read-only Admin API batch Candidate Review application plan for selected review IDs.
  Validation: `cargo test -p nako-server admin_v1_metadata_candidate_review_batch_plan_is_bounded_redacted_and_read_only -- --nocapture`; `cargo test -p nako-api admin_contract -- --nocapture`; `cargo test -p nako-server metadata_candidate_review -- --nocapture`; `cargo test -p nako-metadata candidate_review_application -- --nocapture`; `cargo fmt --all -- --check`; `git diff --check`.
  Review: plan route must do no writes, reuse single-review planning semantics, bound selection size, and redact raw provider/local/secret/idempotency facts.
  Evidence: `EVIDENCE_AND_GATES.md` (`PGBR-020 Evidence`).
  Context: `CONTEXT.jsonl`.
  Handoff: DONE.

## M2 - Confirmed Backend Batch Apply

- [x] PGBR-030 [owner=codex] [deps=PGBR-020] [scope=crates/nako-metadata,crates/nako-api,crates/nako-server,docs/workstreams/provider-governance-bulk-review]
  Goal: Add bounded Admin API batch confirmation that applies only eligible accepted reviews through the existing single-review application service.
  Validation: `cargo test -p nako-server admin_v1_metadata_candidate_review_batch_apply_reports_partial_results_redacted -- --nocapture`; `cargo test -p nako-api admin_contract -- --nocapture`; `cargo test -p nako-server metadata_candidate_review -- --nocapture`; `cargo test -p nako-metadata candidate_review_application -- --nocapture`; `cargo fmt --all -- --check`; `git diff --check`.
  Review: each row must preserve stale guard, idempotency-key fingerprinting, replay behavior, redacted partial results, and root-only Provider Subject / Provider Mapping mutation.
  Evidence: `EVIDENCE_AND_GATES.md` (`PGBR-030 Evidence`).
  Context: `CONTEXT.jsonl`.
  Handoff: DONE.

## M3 - Web Admin Batch Governance

- [x] PGBR-040 [owner=codex] [deps=PGBR-030] [scope=web/src/api/admin,web/src/features/admin,web/src/shell,web/src/test,web/scripts,docs/workstreams/provider-governance-bulk-review]
  Goal: Add Web Admin selection, read-only batch plan inspection, and explicit confirmation from the global Candidate Review queue.
  Validation: `npm --prefix web run check`; `npm --prefix web run test`; `npm --prefix web run build:budget`; browser smoke; `git diff --check`.
  Review: Web must present batch governance as explicit operator confirmation, not fixture-only success, hidden apply, hierarchy mutation, or Public Client API behavior.
  Evidence: `EVIDENCE_AND_GATES.md` (`PGBR-040 Evidence`).
  Context: `CONTEXT.jsonl`.
  Handoff: DONE.

## M4 - Closeout And Follow-On Split

- [ ] PGBR-050 [owner=planner] [deps=PGBR-040] [scope=docs/workstreams/provider-governance-bulk-review,docs/architecture,docs/GOALS.md,docs/ROADMAP.md]
  Goal: Close the lane or split durable job execution, related hierarchy application, provider endpoint depth, and broader provider governance follow-ons.
  Validation: fresh gate evidence in `EVIDENCE_AND_GATES.md`; JSON/JSONL validation; `git diff --check`.
  Review: batch governance must not weaken single-review semantics or leak raw provider/local/secret facts.
  Evidence: `EVIDENCE_AND_GATES.md`; `WORKSTREAM.json`; optional `CLOSEOUT.md`.
  Context: `CONTEXT.jsonl`.
  Handoff: READY.
