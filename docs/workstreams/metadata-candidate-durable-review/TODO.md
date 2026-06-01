# Metadata Candidate Durable Review - TODO

Status: Active
Last updated: 2026-06-02

## M0 - Lane Opening

- [x] MCDR-010 [owner=planner] [deps=none] [scope=docs/workstreams/metadata-candidate-durable-review,docs/architecture,docs/workstreams/README.md,docs/GOALS.md,docs/ROADMAP.md]
  Goal: Open the Metadata Candidate Durable Review lane from provider-depth follow-ons.
  Validation: `python -m json.tool docs/workstreams/metadata-candidate-durable-review/WORKSTREAM.json`; JSONL validation for `TASKS.jsonl`, `CAMPAIGNS.jsonl`, and `CONTEXT.jsonl`; `git diff --check`.
  Evidence: `DESIGN.md`, `WORKSTREAM.json`, `CONTEXT.jsonl`.
  Handoff: Execution begins at `MCDR-020`.

## M1 - Redaction-Safe Review Plan Contract

- [x] MCDR-020 [owner=codex] [deps=MCDR-010] [scope=crates/nako-core/src/media/candidate.rs,crates/nako-metadata/src,crates/nako-metadata/src/tests.rs,docs/workstreams/metadata-candidate-durable-review]
  Goal: Define a provider-neutral Metadata Candidate Review plan contract from `MetadataCandidateGraph` without schema or Provider Mapping writes.
  Validation: `cargo nextest run -p nako-metadata candidate_review metadata_candidate --no-fail-fast`; `cargo fmt --all -- --check`; `git diff --check`.
  Review: review plan includes root and related Provider Subject summaries, relationships, and safe metadata summaries without raw provider payloads or accepted Provider Mapping mutations.
  Evidence: `EVIDENCE_AND_GATES.md`.
  Handoff: DONE; continue to `MCDR-030` planner review before schema.

## M2 - Durable Review Repository Shape

- [x] MCDR-030 [owner=codex] [deps=MCDR-020] [scope=crates/nako-core,crates/nako-db,docs/workstreams/metadata-candidate-durable-review]
  Goal: Decide and implement the durable repository/schema boundary for candidate review snapshots.
  Validation: `cargo nextest run -p nako-db candidate_review --no-fail-fast`; `cargo nextest run -p nako-db provider_subjects --no-fail-fast`; `cargo nextest run -p nako-db baseline_migration --no-fail-fast`; `cargo fmt --all -- --check`; `git diff --check`.
  Review: retention, idempotency, stale-review invalidation, and redaction boundaries are explicit.
  Evidence: `EVIDENCE_AND_GATES.md`.
  Handoff: DONE; continue to `MCDR-040`.

## M3 - Idempotent Accept/Reject Backend Semantics

- [x] MCDR-040 [owner=codex] [deps=MCDR-030] [scope=crates/nako-core,crates/nako-metadata,crates/nako-db,docs/workstreams/metadata-candidate-durable-review]
  Goal: Add backend-only accept/reject semantics for durable candidate reviews before Admin/Web mutation.
  Validation: `cargo nextest run -p nako-metadata candidate_review_decision --no-fail-fast`; `cargo nextest run -p nako-db candidate_review --no-fail-fast`; `cargo fmt --all -- --check`; `git diff --check`.
  Review: accepting or rejecting the same review is idempotent and does not bypass existing Provider Mapping confirmation boundaries.
  Evidence: `EVIDENCE_AND_GATES.md`.
  Handoff: DONE; continue to `MCDR-050`.

## M4 - Closeout

- [ ] MCDR-050 [owner=planner] [deps=MCDR-040] [scope=docs/workstreams/metadata-candidate-durable-review,docs/architecture,docs/GOALS.md,docs/ROADMAP.md]
  Goal: Close the lane or split Admin/Web provider depth governance.
  Validation: fresh gate evidence in `EVIDENCE_AND_GATES.md`; JSON/JSONL validation; `git diff --check`.
  Review: candidate review is durable, redaction-safe, and separate from automatic refresh and Generated Artifact apply.
  Evidence: `CLOSEOUT.md` if closed.
  Handoff: DONE or explicit follow-on split.
