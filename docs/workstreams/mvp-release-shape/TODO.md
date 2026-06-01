# MVP Release Shape - TODO

Status: Active
Last updated: 2026-06-01

## M0 - Scope And Evidence Freeze

- [x] MRS-010 [owner=planner] [deps=none] [scope=docs/workstreams/mvp-release-shape,docs/architecture,docs/GOALS.md,docs/ROADMAP.md]
  Goal: Open the MVP release convergence lane, define the initial MVP cut, and
  link it from planner-owned architecture docs.
  Validation: `python -m json.tool docs/workstreams/mvp-release-shape/WORKSTREAM.json`; `git diff --check -- docs/workstreams/mvp-release-shape docs/architecture/LANES.md docs/architecture/WORKSTREAM_LINKS.md docs/workstreams/README.md docs/GOALS.md docs/ROADMAP.md`.
  Evidence: `MVP.md`, `RELEASE_CUT.md`, `GAP_MATRIX.md`, and `EVIDENCE_AND_GATES.md`.
  Context: `CONTEXT.jsonl`.
  Handoff: DONE. This task created the release-shape planning lane.

## M1 - Release Cut Verification

- [x] MRS-020 [owner=planner] [deps=MRS-010] [scope=docs/workstreams/mvp-release-shape,docs/architecture,docs/workstreams]
  Goal: Verify P0/P1/P2 release cut rows against current code, docs, active
  workstreams, known gates, and related repo status.
  Validation: `python -m json.tool docs/workstreams/mvp-release-shape/WORKSTREAM.json`; `git diff --check -- docs/workstreams/mvp-release-shape`.
  Review: Planner review for whether each P0 requirement is evidence-backed,
  blocked, or deferred.
  Evidence: Updated `GAP_MATRIX.md` and `EVIDENCE_AND_GATES.md`.
  Context: `CONTEXT.jsonl`.
  Handoff: DONE_WITH_CONCERNS. Verification found strong evidence for install,
  scan, metadata, storage, Admin diagnostics, Addon Sidecars, and remote-access
  cookbook scope, but `PTJCH-220` remains a P0 blocker and `MRS-030` must turn
  the release cut into fresh smoke/gate evidence.

## M2 - MVP Gate Plan

- [x] MRS-030 [owner=planner] [deps=MRS-020] [scope=docs/workstreams/mvp-release-shape,docs/deployment,docs/architecture]
  Goal: Define the MVP validation ladder and route missing release gates to
  existing or new workstreams.
  Validation: `git diff --check -- docs/workstreams/mvp-release-shape docs/deployment docs/architecture`.
  Review: Review for whether gates prove the required user journey rather than
  only package-local behavior.
  Evidence: Updated `EVIDENCE_AND_GATES.md` and follow-on links.
  Context: `CONTEXT.jsonl`.
  Handoff: DONE_WITH_CONCERNS. `EVIDENCE_AND_GATES.md` now contains the MVP
  validation ladder. `PTJCH-220` remains the P0 playback blocker, and the
  Web/Public Client live smoke should split to `web-product` if existing tests
  plus manual browser smoke are not reproducible enough for release.

## M3 - Active Queue Alignment

- [x] MRS-040 [owner=planner] [deps=MRS-020] [scope=docs/architecture/LANES.md,docs/workstreams/*]
  Goal: Align `PTJCH`, `GAMA`, and `CSAPA` with the MVP release cut: finish,
  split, or explicitly defer each active tail.
  Validation: workstream inventory has no readiness drift for MVP-blocking
  active tasks.
  Review: Use `integrate-lane-results` for worker completions before marking
  blockers resolved.
  Evidence: `GAP_MATRIX.md`, `HANDOFF.md`, and affected workstream evidence.
  Context: `CONTEXT.jsonl`.
  Handoff: DONE_WITH_CONCERNS. Active queue is aligned for MVP: `PTJCH-220`
  remains the only current active-tail P0 blocker and has a parallel playback
  worker running; `GAMA-060` is conditional/P1; `CSAPA-050` is deferred/P1.
  Do not mutate those active task ledgers from the MVP planner lane.

## M4 - Closeout Or Campaign Split

- [ ] MRS-050 [owner=planner] [deps=MRS-030,MRS-040] [scope=docs/workstreams/mvp-release-shape,docs/GOALS.md,docs/ROADMAP.md]
  Goal: Close this planning lane or split a small set of MVP implementation
  campaigns with exact owners, worktrees, gates, and side-effect policies.
  Validation: `python -m json.tool docs/workstreams/mvp-release-shape/WORKSTREAM.json`; `git diff --check`.
  Review: `review-workstream` for workstream compliance before closeout.
  Evidence: `CAMPAIGNS.md`, `EVIDENCE_AND_GATES.md`, `HANDOFF.md`, and
  optional `CLOSEOUT.md`.
  Context: `CONTEXT.jsonl`.
  Handoff: IN_PROGRESS. Campaign A (`PTJCH-220`) and Campaign B
  (`web-mvp-live-smoke`) are integrated on `main`; Campaign C remains optional
  for one-command release proof, and Campaign D remains conditional on an
  official addon claim. Release-candidate Gate 0, Gate 1, and Gate 2 now pass;
  continue from Gate 3 Web/Public Client validation.
