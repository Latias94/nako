# Architecture Roadmap Reconciliation - Evidence And Gates

Status: Active
Last updated: 2026-06-01

## Always Run

- `python -m json.tool docs/workstreams/architecture-roadmap-reconciliation/WORKSTREAM.json`
- `git diff --check -- docs/GOALS.md docs/ROADMAP.md docs/architecture docs/workstreams/README.md docs/workstreams/architecture-roadmap-reconciliation`

## Targeted Stale-Reference Checks

Run after `ARR-030` or `ARR-040` depending on which files changed:

- `rg -n "docs/adr/0053-runtime-control-plane-boundary.md" docs -g "!docs/workstreams/architecture-roadmap-reconciliation/EVIDENCE_AND_GATES.md"`
- `rg -n "Douban provider \\| Not started|Bangumi provider \\| Not started|tmdb-series-season-episode-depth|douban-provider-mvp|bangumi-provider-mvp" docs/architecture`
- `rg -n "No planner-owned architecture focus is currently open|No top-level planner goal is currently open" docs/GOALS.md docs/ROADMAP.md`

## Evidence Log

- `ARR-010`: Opened this workstream after six read-only sub-architecture
  audits returned `DONE_WITH_CONCERNS`.
- `ARR-020`: Updated `docs/GOALS.md`, `docs/ROADMAP.md`,
  `docs/architecture/LANES.md`, and `docs/workstreams/README.md` so the active
  queue points at this planner lane and recent completed lanes are not shown as
  active work.
- `ARR-030`: Updated `WORKSTREAM_LINKS.md`, `LIBRARY_PIPELINE.md`,
  `STATE_ACCESS.md`, and `CONTROL_PLANE.md` for shipped provider, artwork,
  playback policy, cache/header, Web, addon, realtime, and control-plane
  evidence.
- `ARR-040`: Repaired high-risk stale references that could misroute future
  work, including the old ADR 0053 path and historical handoff wording that
  contradicted later closed lanes.

## Notes

This is a docs-only planner lane. Rust, Web, schema, generated-contract, and
runtime tests are not required unless the scope changes.
