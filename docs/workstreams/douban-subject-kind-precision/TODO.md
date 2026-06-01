# Douban Subject Kind Precision - TODO

Status: Active
Last updated: 2026-06-02

## M0 - Lane Opening

- [x] DSKP-010 [owner=planner] [deps=none] [scope=docs/workstreams/douban-subject-kind-precision,docs/architecture,docs/workstreams/README.md,docs/GOALS.md,docs/ROADMAP.md]
  Goal: Open the Douban subject-kind precision lane from provider-depth follow-ons.
  Validation: `python -m json.tool docs/workstreams/douban-subject-kind-precision/WORKSTREAM.json`; JSONL validation for `TASKS.jsonl`, `CAMPAIGNS.jsonl`, and `CONTEXT.jsonl`; `git diff --check`.
  Evidence: `DESIGN.md`, `WORKSTREAM.json`, `CONTEXT.jsonl`.
  Handoff: Execution begins at `DSKP-020`.

## M1 - Endpoint-Backed Douban Capability Claims

- [x] DSKP-020 [owner=codex] [deps=DSKP-010] [scope=crates/nako-metadata/src/providers/douban.rs,crates/nako-metadata/src/tests.rs,docs/workstreams/douban-subject-kind-precision]
  Goal: Narrow Douban provider media/subject capability claims to endpoint-backed movie behavior and add regression coverage.
  Validation: `cargo nextest run -p nako-metadata douban_provider built_in_provider_capabilities --no-fail-fast`; `cargo nextest run -p nako-metadata douban_provider metadata_candidate --no-fail-fast`; `cargo fmt --all -- --check`; `git diff --check`.
  Review: no false series/season/episode support remains while only movie endpoints are implemented.
  Evidence: `EVIDENCE_AND_GATES.md`.
  Handoff: DONE; continue to `DSKP-030`.

## M2 - Closeout

- [ ] DSKP-030 [owner=planner] [deps=DSKP-020] [scope=docs/workstreams/douban-subject-kind-precision,docs/architecture,docs/GOALS.md,docs/ROADMAP.md]
  Goal: Close the lane or split future Douban TV/episode endpoint follow-ons.
  Validation: fresh gate evidence in `EVIDENCE_AND_GATES.md`; JSON/JSONL validation; `git diff --check`.
  Review: Douban precision remains endpoint-backed and movie-compatible.
  Evidence: `CLOSEOUT.md` if closed.
  Handoff: DONE or explicit follow-on split.
