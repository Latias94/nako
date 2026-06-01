# TMDB Season Episode Graph Depth — TODO

Status: Active
Last updated: 2026-06-02

## M0 — Lane Opening

- [x] TSEG-010 [owner=planner] [deps=none] [scope=docs/workstreams/tmdb-season-episode-graph-depth,docs/architecture,docs/workstreams/README.md,docs/GOALS.md,docs/ROADMAP.md]
  Goal: Open the TMDB season -> episode graph depth lane from MPDP closeout.
  Validation: `python -m json.tool docs/workstreams/tmdb-season-episode-graph-depth/WORKSTREAM.json`; JSONL validation for `TASKS.jsonl`, `CAMPAIGNS.jsonl`, and `CONTEXT.jsonl`; `git diff --check`.
  Evidence: `DESIGN.md`, `WORKSTREAM.json`, `CONTEXT.jsonl`.
  Handoff: Execution begins at `TSEG-020`.

## M1 — TMDB Season Episode Graph Preview

- [x] TSEG-020 [owner=codex] [deps=TSEG-010] [scope=crates/nako-metadata/src/providers/tmdb.rs,crates/nako-metadata/src/mapping/tmdb.rs,crates/nako-metadata/src/tests.rs,docs/workstreams/tmdb-season-episode-graph-depth]
  Goal: Add TMDB season -> episode provider graph preview without hierarchy or mapping mutations.
  Validation: `cargo nextest run -p nako-metadata tmdb_provider_supports_series_season_and_episode_fetches --no-fail-fast`; `cargo nextest run -p nako-metadata tmdb_provider metadata_candidate --no-fail-fast`; `cargo fmt --all -- --check`; `git diff --check`.
  Review: episode nodes are evidence only; root season Canonical Metadata stays unchanged. Passed on 2026-06-02.
  Evidence: `EVIDENCE_AND_GATES.md` TSEG-020 section.
  Handoff: Continue to `TSEG-030` after graph preview tests pass.

## M2 — Root-Only Season Refresh Guard

- [ ] TSEG-030 [owner=codex] [deps=TSEG-020] [scope=crates/nako-metadata/src/tests.rs,docs/workstreams/tmdb-season-episode-graph-depth]
  Goal: Prove TMDB episode graph preview remains non-mutating during season refresh.
  Validation: focused `cargo nextest run -p nako-metadata refresh season metadata_candidate --no-fail-fast`; `cargo fmt --all -- --check`; `git diff --check`.
  Review: no automatic episode Media Item creation, no episode Provider Subject insertion, no child Provider Mapping writes.
  Evidence: `EVIDENCE_AND_GATES.md`.
  Handoff: Continue to `TSEG-040`.

## M3 — Closeout

- [ ] TSEG-040 [owner=planner] [deps=TSEG-030] [scope=docs/workstreams/tmdb-season-episode-graph-depth,docs/architecture,docs/GOALS.md,docs/ROADMAP.md]
  Goal: Close the lane or split durable review/Admin confirmation follow-ons.
  Validation: fresh gate evidence in `EVIDENCE_AND_GATES.md`; JSON/JSONL validation; `git diff --check`.
  Review: TMDB episode graph depth remains preview-only.
  Evidence: `CLOSEOUT.md` if closed.
  Handoff: DONE or explicit follow-on split.
