# Bangumi Relations And Episode Depth - TODO

Status: Active
Last updated: 2026-06-02

## M0 - Lane Opening

- [x] BRED-010 [owner=planner] [deps=none] [scope=docs/workstreams/bangumi-relations-and-episode-depth,docs/architecture,docs/workstreams/README.md,docs/GOALS.md,docs/ROADMAP.md]
  Goal: Open the Bangumi relation and episode depth lane from provider-depth follow-ons.
  Validation: `python -m json.tool docs/workstreams/bangumi-relations-and-episode-depth/WORKSTREAM.json`; JSONL validation for `TASKS.jsonl`, `CAMPAIGNS.jsonl`, and `CONTEXT.jsonl`; `git diff --check`.
  Evidence: `DESIGN.md`, `WORKSTREAM.json`, `CONTEXT.jsonl`.
  Handoff: Execution begins at `BRED-020`.

## M1 - Endpoint-Backed Capability Claims

- [x] BRED-020 [owner=codex] [deps=BRED-010] [scope=crates/nako-metadata/src/providers/bangumi.rs,crates/nako-metadata/src/tests.rs,docs/workstreams/bangumi-relations-and-episode-depth]
  Goal: Narrow Bangumi provider media/subject capability claims to endpoint-backed behavior and add regression coverage.
  Validation: `cargo nextest run -p nako-metadata bangumi_provider metadata_candidate --no-fail-fast`; `cargo fmt --all -- --check`; `git diff --check`.
  Review: no false season/episode support remains before episode endpoints are implemented. Passed on 2026-06-02.
  Evidence: `EVIDENCE_AND_GATES.md` BRED-020 section.
  Handoff: Continue to `BRED-030`.

## M2 - Bangumi Episode Graph Preview

- [x] BRED-030 [owner=codex] [deps=BRED-020] [scope=crates/nako-metadata/src/providers/bangumi.rs,crates/nako-metadata/src/mapping/bangumi.rs,crates/nako-metadata/src/tests.rs,docs/workstreams/bangumi-relations-and-episode-depth]
  Goal: Add endpoint-backed Bangumi episode graph preview for series fetches without persistence changes.
  Validation: `cargo nextest run -p nako-metadata bangumi_provider metadata_candidate --no-fail-fast`; `cargo fmt --all -- --check`; `git diff --check`.
  Review: episode nodes are evidence only and use precise Provider Subject keys. Passed on 2026-06-02.
  Evidence: `EVIDENCE_AND_GATES.md` BRED-030 section.
  Handoff: Continue to `BRED-040`.

## M3 - Root-Only Refresh Guard

- [x] BRED-040 [owner=codex] [deps=BRED-030] [scope=crates/nako-metadata/src/tests.rs,docs/workstreams/bangumi-relations-and-episode-depth]
  Goal: Prove Bangumi episode graph preview remains non-mutating during refresh.
  Validation: `cargo nextest run -p nako-metadata bangumi refresh metadata_candidate --no-fail-fast`; `cargo fmt --all -- --check`; `git diff --check`.
  Review: no automatic episode Media Item creation, no episode Provider Subject insertion, no child Provider Mapping writes. Passed on 2026-06-02.
  Evidence: `EVIDENCE_AND_GATES.md` BRED-040 section.
  Handoff: Continue to `BRED-050`.

## M4 - Closeout

- [ ] BRED-050 [owner=planner] [deps=BRED-040] [scope=docs/workstreams/bangumi-relations-and-episode-depth,docs/architecture,docs/GOALS.md,docs/ROADMAP.md]
  Goal: Close the lane or split durable review/Admin confirmation follow-ons.
  Validation: fresh gate evidence in `EVIDENCE_AND_GATES.md`; JSON/JSONL validation; `git diff --check`.
  Review: Bangumi depth remains preview-only.
  Evidence: `CLOSEOUT.md` if closed.
  Handoff: DONE or explicit follow-on split.
