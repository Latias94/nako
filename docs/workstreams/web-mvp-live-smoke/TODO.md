# Web MVP Live Smoke - TODO

Status: Active
Last updated: 2026-06-01

## M0 - Scope And Evidence Freeze

- [x] WMLS-010 [owner=web-product] [deps=none] [scope=docs/workstreams/web-mvp-live-smoke,docs/architecture]
  Goal: Open the Web MVP live smoke lane from MVP Campaign B with lane slug,
  context manifest, gate set, and architecture links.
  Validation: `python -m json.tool docs/workstreams/web-mvp-live-smoke/WORKSTREAM.json`
  Evidence: `docs/workstreams/web-mvp-live-smoke/DESIGN.md`
  Context: `docs/workstreams/web-mvp-live-smoke/CONTEXT.jsonl`
  Handoff: Continue to WMLS-020 in the same clean worktree.

## M1 - Dedicated Web MVP Smoke

- [x] WMLS-020 [owner=web-product] [deps=WMLS-010] [scope=web/src/test]
  Goal: Add a dedicated smoke that checks the MVP Web/Public Client route and
  playback path without expanding product scope.
  Validation: `npm --prefix web run test -- src/test/mvp-live-smoke.test.tsx`
  Review: Review that the smoke covers `/media`, `/media/library`,
  `/media/detail`, browser playback tickets, native video/subtitle rendering,
  heartbeat through `playback_session_id`, and no token/raw-path exposure.
  Evidence: `web/src/test/mvp-live-smoke.test.tsx`
  Context: `docs/workstreams/web-mvp-live-smoke/CONTEXT.jsonl`
  Handoff: Final status must be `DONE`, `DONE_WITH_CONCERNS`, `BLOCKED`, or
  `NEEDS_CONTEXT`.

## M2 - Release Evidence Verification

- [x] WMLS-030 [owner=web-product] [deps=WMLS-020] [scope=docs/workstreams/web-mvp-live-smoke,web]
  Goal: Run the required Web gates and record fresh release evidence.
  Validation: `npm --prefix web run test`; `npm --prefix web run check`;
  `npm --prefix web run build:budget`; `git diff --check -- docs/workstreams/web-mvp-live-smoke web`
  Review: `integrate-lane-results` should review the worker report before
  accepting this lane as MVP Gate 3 evidence.
  Evidence: `docs/workstreams/web-mvp-live-smoke/EVIDENCE_AND_GATES.md`
  Context: `docs/workstreams/web-mvp-live-smoke/CONTEXT.jsonl`
  Handoff: Full Web gates passed on 2026-06-01; return to planner integration.

## M3 - Closeout

- [ ] WMLS-040 [owner=planner] [deps=WMLS-030] [scope=docs/workstreams/web-mvp-live-smoke,docs/workstreams/mvp-release-shape]
  Goal: Close this lane or split only proven follow-ons.
  Validation: Fresh Web gate evidence exists and the MVP planner can link Gate 3
  to this workstream.
  Review: Close through `review-workstream` or planner integration.
  Evidence: `docs/workstreams/web-mvp-live-smoke/EVIDENCE_AND_GATES.md`
  Context: `docs/workstreams/web-mvp-live-smoke/CONTEXT.jsonl`
  Handoff: Return any follow-on to the planner; do not self-assign desktop,
  backend, or contract work.
