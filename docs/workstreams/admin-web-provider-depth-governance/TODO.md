# Admin Web Provider Depth Governance - TODO

Status: Active
Last updated: 2026-06-02

## M0 - Lane Opening

- [x] AWPDG-010 [owner=planner] [deps=none] [scope=docs/workstreams/admin-web-provider-depth-governance,docs/architecture,docs/workstreams/README.md,docs/GOALS.md,docs/ROADMAP.md]
  Goal: Open the Admin/Web provider depth governance lane split from ARPMA closeout.
  Validation: `python -m json.tool docs/workstreams/admin-web-provider-depth-governance/WORKSTREAM.json`; JSONL validation for `TASKS.jsonl`, `CAMPAIGNS.jsonl`, and `CONTEXT.jsonl`; `git diff --check`.
  Evidence: `DESIGN.md`, `WORKSTREAM.json`, `CONTEXT.jsonl`.
  Handoff: Execution begins at `AWPDG-020`.

## M1 - Read-Only Admin API Review Plan

- [ ] AWPDG-020 [owner=codex] [deps=AWPDG-010] [scope=crates/nako-api,crates/nako-server,crates/nako-metadata,docs/workstreams/admin-web-provider-depth-governance]
  Goal: Expose durable Metadata Candidate Review detail and accepted-review application plan facts through a redaction-safe read-only Admin API boundary.
  Validation: `cargo nextest run -p nako-api admin_contract --no-fail-fast`; `cargo nextest run -p nako-server candidate_review admin --no-fail-fast`; `cargo fmt --all -- --check`; `git diff --check`.
  Review: route must not write Provider Subject, Provider Mapping, Canonical Metadata, or related graph node state.
  Evidence: `EVIDENCE_AND_GATES.md`.
  Handoff: Continue to `AWPDG-030`.

## M2 - Confirmed Admin API Apply Mutation

- [ ] AWPDG-030 [owner=codex] [deps=AWPDG-020] [scope=crates/nako-api,crates/nako-server,crates/nako-metadata,docs/workstreams/admin-web-provider-depth-governance]
  Goal: Add an explicit Admin API mutation that applies an accepted review through `MetadataCandidateReviewApplicationService` with stale guards and idempotency.
  Validation: `cargo nextest run -p nako-api admin_contract --no-fail-fast`; `cargo nextest run -p nako-server candidate_review admin --no-fail-fast`; `cargo nextest run -p nako-metadata candidate_review_application --no-fail-fast`; `cargo fmt --all -- --check`; `git diff --check`.
  Review: mutation must apply only root Provider Subject / Provider Mapping state and must make noop/conflict/replay behavior visible.
  Evidence: `EVIDENCE_AND_GATES.md`.
  Handoff: Continue to `AWPDG-040`.

## M3 - Web Admin Governance Surface

- [ ] AWPDG-040 [owner=codex] [deps=AWPDG-030] [scope=web/src/api/admin,web/src/features/admin,web/src/test,docs/workstreams/admin-web-provider-depth-governance]
  Goal: Add Web Admin read/confirm/apply UX for durable Candidate Review evidence and accepted-review application.
  Validation: `npm --prefix web run test`; `npm --prefix web run check`; `npm --prefix web run build:budget`; browser smoke if a route is added; `git diff --check`.
  Review: Web must distinguish preview graph evidence from accepted Provider Mapping facts and must not imply related nodes are applied.
  Evidence: `EVIDENCE_AND_GATES.md`.
  Handoff: Continue to `AWPDG-050`.

## M4 - Closeout And Follow-On Split

- [ ] AWPDG-050 [owner=planner] [deps=AWPDG-040] [scope=docs/workstreams/admin-web-provider-depth-governance,docs/architecture,docs/GOALS.md,docs/ROADMAP.md]
  Goal: Close the lane or split related-node hierarchy application, Douban TV/episode depth, and broader provider governance follow-ons.
  Validation: fresh gate evidence in `EVIDENCE_AND_GATES.md`; JSON/JSONL validation; `git diff --check`.
  Review: Admin/Web can safely govern durable Candidate Review application without broadening Public Client API or hierarchy mutation.
  Evidence: `CLOSEOUT.md` if closed.
  Handoff: DONE or explicit follow-on split.
