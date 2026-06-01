# Accepted Review Provider Mapping Application - TODO

Status: Closed
Last updated: 2026-06-02

## M0 - Lane Opening

- [x] ARPMA-010 [owner=planner] [deps=none] [scope=docs/workstreams/accepted-review-provider-mapping-application,docs/architecture,docs/workstreams/README.md,docs/GOALS.md,docs/ROADMAP.md]
  Goal: Open the accepted-review Provider Mapping application lane from MCDR closeout.
  Validation: `python -m json.tool docs/workstreams/accepted-review-provider-mapping-application/WORKSTREAM.json`; JSONL validation for `TASKS.jsonl`, `CAMPAIGNS.jsonl`, and `CONTEXT.jsonl`; `git diff --check`.
  Evidence: `DESIGN.md`, `WORKSTREAM.json`, `CONTEXT.jsonl`.
  Handoff: Execution begins at `ARPMA-020`.

## M1 - Read-Only Application Plan

- [x] ARPMA-020 [owner=codex] [deps=ARPMA-010] [scope=crates/nako-core/src/media/candidate.rs,crates/nako-metadata/src/candidate_review.rs,crates/nako-metadata/src/tests.rs,docs/workstreams/accepted-review-provider-mapping-application]
  Goal: Define a read-only accepted-review Provider Mapping application plan with action/reason/source conversion semantics and no Provider Mapping writes.
  Validation: `cargo nextest run -p nako-metadata candidate_review_application --no-fail-fast`; `cargo nextest run -p nako-metadata --no-fail-fast`; `cargo nextest run -p nako-core --no-fail-fast`; `cargo fmt --all -- --check`; `git diff --check`.
  Review: plan reasons explain accepted/not-accepted, missing root subject, unsupported source, existing accepted/rejected mapping, noop, and ready behavior.
  Evidence: `EVIDENCE_AND_GATES.md`.
  Handoff: DONE; continue to `ARPMA-030`.

## M2 - Root Provider Mapping Apply Service

- [x] ARPMA-030 [owner=codex] [deps=ARPMA-020] [scope=crates/nako-core,crates/nako-metadata,crates/nako-db,docs/workstreams/accepted-review-provider-mapping-application]
  Goal: Apply accepted candidate review root Provider Subject and Provider Mapping idempotently through existing repository semantics.
  Validation: `cargo nextest run -p nako-metadata candidate_review_application --no-fail-fast`; `cargo nextest run -p nako-metadata --no-fail-fast`; `cargo nextest run -p nako-db candidate_review provider_mapping --no-fail-fast`; `cargo fmt --all -- --check`; `git diff --check`.
  Review: application writes only the root Provider Subject/Mapping, protects rejected mappings, and never applies related graph nodes.
  Evidence: `EVIDENCE_AND_GATES.md`.
  Handoff: DONE; continue to `ARPMA-040`.

## M3 - Surface Split Review

- [x] ARPMA-040 [owner=planner] [deps=ARPMA-030] [scope=docs/workstreams/accepted-review-provider-mapping-application,docs/architecture,docs/GOALS.md,docs/ROADMAP.md]
  Goal: Decide whether an Admin API route belongs in this lane or should split with Admin/Web provider depth governance.
  Validation: fresh evidence in `EVIDENCE_AND_GATES.md`; JSON/JSONL validation; `git diff --check`.
  Review: split Admin API/Web mutation scope to `proposed:admin-web-provider-depth-governance`; backend application semantics are safe, but product exposure needs its own operator workflow.
  Evidence: `EVIDENCE_AND_GATES.md`.
  Handoff: DONE; continue to `ARPMA-050` closeout.

## M4 - Closeout

- [x] ARPMA-050 [owner=planner] [deps=ARPMA-040] [scope=docs/workstreams/accepted-review-provider-mapping-application,docs/architecture,docs/GOALS.md,docs/ROADMAP.md]
  Goal: Close the lane or split Admin/Web governance and related-node hierarchy follow-ons.
  Validation: fresh gate evidence in `EVIDENCE_AND_GATES.md`; JSON/JSONL validation; `git diff --check`.
  Review: accepted reviews can be safely planned/applied without hidden refresh or UI side effects.
  Evidence: `CLOSEOUT.md` if closed.
  Handoff: DONE; open `proposed:admin-web-provider-depth-governance` when product/API exposure begins.
