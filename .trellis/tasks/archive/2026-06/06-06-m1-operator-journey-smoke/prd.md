# M1 Operator Journey Smoke

## Goal

Create the first executable Product-Operator M1 smoke slice. It should prove,
with the smallest useful implementation, that Nako can demonstrate the selected
M1 journey from one configured Media Library through scan/index visibility,
catalog or media browse, playback readiness, and Admin diagnostics/repair
surfaces.

This task is the first implementation step after roadmap reconciliation. It is
not the whole M1 release. Its job is to turn the roadmap queue into a concrete
gate that future M1 tasks can run and improve.

## What I Already Know

- M1 is Product-Operator first: one self-hosted, video-first,
  single-admin/operator journey.
- The backend release ladder remains the M1 quality gate, not the product
  definition.
- Existing server smoke coverage is available through
  `cargo nextest run -p nako-server self_host_smoke --no-fail-fast` and
  `scripts/self-host-smoke.ps1`.
- Existing release-gate coverage is available through
  `scripts/release-gate.ps1`.
- Admin Web already has deterministic route/media tests for library
  management, scan command visibility, media browse/playback surfaces,
  playback sessions, storage diagnostics, catalog governance, and several
  redaction-sensitive routes.
- The old `web-mvp-live-smoke` workstream proves public/media browse and
  browser playback behavior as historical evidence, but the current M1 queue
  needs a focused Trellis task rather than reopening a workstream.

## Assumptions

- The smallest valuable slice is a named, repeatable smoke artifact or test
  grouping that composes existing coverage and fills only the most obvious
  M1 journey gap.
- Existing APIs and generated contracts should be reused where possible.
- If the implementation discovers a missing API or generated contract change,
  it should stop and narrow the task rather than expanding silently.
- A deterministic local test is preferable to a live browser/manual smoke for
  this first slice unless existing test coverage cannot express the journey.

## Requirements

- Define a clear M1 operator journey smoke artifact, runbook, script, or
  focused test path that future agents can execute.
- Cover or explicitly link the journey steps:
  - one Media Library is configured and visible;
  - scan/index work can be requested or observed;
  - media/catalog browse exposes entries or source inventory;
  - playback readiness is visible through an existing Direct Play, Remux, or
    HLS decision/ticket/session path;
  - Admin diagnostics and repair surfaces remain available and redaction-safe.
- Prefer reuse of existing `self_host_smoke`, Admin Web media route tests,
  `scripts/self-host-smoke.ps1`, and `scripts/release-gate.ps1` behavior.
- Keep generated Admin/Public Client contract changes out of scope unless a
  small missing contract is unavoidable and fully regenerated/tested.
- Record evidence that the smoke does not expose tokens, local paths, raw
  source locators, playback output paths, database URLs, or other sensitive
  facts in checked UI/API surfaces.
- If the smoke reveals gaps that are larger than this slice, document them as
  follow-on Trellis task candidates rather than absorbing them.

## Acceptance Criteria

- [ ] A repeatable M1 operator journey smoke entry point or focused test path
      exists and is documented in this task evidence or project docs.
- [ ] The smoke maps each Product-Operator M1 step to concrete existing or new
      checks: library config, scan/index, browse, playback, diagnostics/repair,
      and redaction.
- [ ] Existing historical evidence remains linked and is not deleted.
- [ ] Any code changes are limited to the smallest required server/Admin Web
      smoke or support code.
- [ ] No schema, release artifact publication, broad route redesign, or hidden
      runtime behavior is introduced.
- [ ] Focused server/Admin Web gates pass for touched areas.
- [ ] `python ./.trellis/scripts/task.py validate 06-06-m1-operator-journey-smoke`
      passes.
- [ ] `git diff --check` passes for touched files.

## Definition Of Done

- Implementation and independent check agents complete.
- Focused gates for changed server/Admin Web/script/docs areas pass.
- The task evidence records commands run, results, changed files, and residual
  gaps.
- Spec update is completed or explicitly judged unnecessary.
- Work is committed, task is archived, and session progress is recorded.

## Out Of Scope

- Full M1 release execution or artifact publication.
- New workstream directories.
- Automatic source hash triggering after scan.
- Automatic source duplicate merge or operator source duplicate flow.
- Broad Media Web player UX redesign.
- Addon Manager or official Addon Sidecar proof.
- Public Client metadata governance, mobile/TV client breadth, LL-HLS/CMAF,
  or metadata undo.

## Technical Approach

Start from existing evidence and make the first M1 smoke an integration point
rather than a new product surface. The likely implementation should either:

- add a small script/runbook that composes the existing server and Admin Web
  focused gates; or
- add a narrow deterministic test that connects existing Admin Web/media route
  fixtures into one Product-Operator M1 journey assertion; or
- do both if the missing implementation is small and the gates remain focused.

The implementation should search for current tests before adding new ones and
should prefer extending existing smoke fixtures over duplicating route,
playback, or diagnostics setup.

## Decision

Use the **composed smoke artifact** approach for this first slice. The first
M1 task should prove journey alignment by composing and tightening existing
gates, not by inventing new runtime behavior.

Consequences:

- Fast to validate and low risk.
- Provides a clear target for later M1 tasks.
- May expose that some journey steps are currently evidenced by separate tests;
  those gaps should become follow-on tasks if they require product work.

## Research References

- `research/operator-journey-smoke-assets.md` - current smoke, release-gate,
  Admin Web, and historical MVP evidence inventory.

## Technical Notes

- Relevant roadmap anchor:
  `docs/ROADMAP.md` M1 Release Convergence Queue.
- Relevant lane routing:
  `docs/architecture/LANES.md` Active Queue.
- Relevant historical evidence:
  `docs/workstreams/mvp-release-shape/CLOSEOUT.md` and
  `docs/workstreams/web-mvp-live-smoke/EVIDENCE_AND_GATES.md`.
- Relevant specs:
  `.trellis/spec/admin-web/frontend/routes-forms-and-tests.md`,
  `.trellis/spec/nako-server/backend/quality-guidelines.md`,
  `.trellis/spec/guides/cross-layer-thinking-guide.md`, and
  `.trellis/spec/guides/code-reuse-thinking-guide.md`.
