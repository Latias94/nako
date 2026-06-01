# Metadata Provider Depth And Precision — TODO

Status: Active
Last updated: 2026-06-02

## M0 — Lane Opening

- [x] MPDP-010 [owner=planner] [deps=none] [scope=docs/workstreams/metadata-provider-depth-and-precision,docs/architecture,docs/workstreams/README.md,docs/GOALS.md,docs/ROADMAP.md]
  Goal: Open the provider depth and precision lane after GAARA closeout.
  Validation: `python -m json.tool docs/workstreams/metadata-provider-depth-and-precision/WORKSTREAM.json`; JSONL validation for `TASKS.jsonl`, `CAMPAIGNS.jsonl`, and `CONTEXT.jsonl`; `git diff --check`.
  Evidence: `DESIGN.md`, `WORKSTREAM.json`, `CONTEXT.jsonl`.
  Handoff: Execution begins at `MPDP-020`.

## M1 — TMDB Series Season Graph Preview

- [x] MPDP-020 [owner=codex] [deps=MPDP-010] [scope=crates/nako-core/src/media/candidate.rs,crates/nako-metadata/src/providers/tmdb.rs,crates/nako-metadata/src/mapping/tmdb.rs,crates/nako-metadata/src/tests.rs,docs/workstreams/metadata-provider-depth-and-precision]
  Goal: Add a TMDB series -> season provider graph preview without creating hierarchy or mapping mutations.
  Validation: `cargo nextest run -p nako-metadata tmdb_provider --no-fail-fast`; `cargo nextest run -p nako-metadata matching_policy candidate_conflict_review --no-fail-fast`; `git diff --check`.
  Review: season nodes are evidence only; root Canonical Metadata stays unchanged; no schema change, no Public Client API change, no Generated Artifact apply changes. Passed on 2026-06-02.
  Evidence: `EVIDENCE_AND_GATES.md` MPDP-020 section.
  Handoff: Continue to `MPDP-030` only after graph preview tests pass.

## M2 — Root-Only Refresh Guard

- [x] MPDP-030 [owner=codex] [deps=MPDP-020] [scope=crates/nako-metadata/src/provider_attempt.rs,crates/nako-metadata/src/strategy.rs,crates/nako-metadata/src/tests.rs,crates/nako-server/src/app/tests/metadata.rs]
  Goal: Prove TMDB graph depth remains non-mutating during refresh and only root Provider Mapping behavior persists.
  Validation: focused `cargo nextest run -p nako-metadata metadata_refresh tmdb --no-fail-fast`; focused server metadata refresh tests if persistence behavior changes; `cargo fmt --all -- --check` when Rust changes.
  Review: no automatic Media Item hierarchy creation, no child Provider Mapping writes, external-ID refresh compatibility remains covered. Passed on 2026-06-02.
  Evidence: `EVIDENCE_AND_GATES.md` MPDP-030 section.
  Handoff: Continue to `MPDP-040` for closeout or follow-on split.

## M3 — Follow-On Split

- [x] MPDP-040 [owner=planner] [deps=MPDP-030] [scope=docs/workstreams/metadata-provider-depth-and-precision,docs/architecture,docs/GOALS.md,docs/ROADMAP.md]
  Goal: Split episode graph depth, Bangumi relation depth, Douban precision, durable candidate review, and Admin/Web confirmation follow-ons.
  Validation: fresh gate evidence in `EVIDENCE_AND_GATES.md`; JSON/JSONL validation; `git diff --check`.
  Review: first slice remains TMDB season graph preview only. Passed on 2026-06-02.
  Evidence: `FOLLOW_ONS.md`, `EVIDENCE_AND_GATES.md` MPDP-040 section.
  Handoff: Continue to `MPDP-050`.

## M4 — Closeout

- [ ] MPDP-050 [owner=planner] [deps=MPDP-040] [scope=docs/workstreams/metadata-provider-depth-and-precision,docs/architecture,docs/GOALS.md,docs/ROADMAP.md]
  Goal: Close the lane or split durable candidate review, Admin governance detail, and Web confirmation follow-ons.
  Validation: fresh gate evidence in `EVIDENCE_AND_GATES.md`; JSON/JSONL validation; `git diff --check`.
  Review: provider depth claims are honest, tests cover the shipped precision, and follow-ons are split.
  Evidence: `CLOSEOUT.md` if closed.
  Handoff: DONE or explicit follow-on split.
