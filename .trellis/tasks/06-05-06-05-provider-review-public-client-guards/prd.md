# Provider Review Public Client Governance Map Reconciliation

## Goal

Reconcile active roadmap and architecture maps after
`42869eaf test(api): guard provider governance public contracts` shipped the
negative Public Client governance guardrails.

## Requirements

- Remove `proposed:provider-review-public-client-governance` from active
  proposed follow-on queues.
- Mark the shipped guardrails as completed evidence where metadata governance
  follow-ons are summarized.
- Preserve true remaining follow-ons:
  - Douban Season/Episode graph depth;
  - provider governance mutation-capable undo;
  - provider identity mapping breadth;
  - any future intentional Public Client metadata exposure as a separate,
    explicit API-design PRD, not this completed guardrail label.
- Do not change Rust code, API behavior, generated SDKs, or specs.

## Acceptance Criteria

- [x] Focused grep finds no `proposed:provider-review-public-client-governance`
      in active docs.
- [x] Active docs reference `42869eaf` or the archived task as shipped evidence.
- [x] Remaining follow-ons stay visible and narrowly named.
- [x] `git diff --check` and Trellis task validation pass.

## Definition Of Done

- Docs-only diff.
- Task evidence records the grep/validation commands.
- Task is committed and archived.

## Validation Evidence

- `rg -n "proposed:provider-review-public-client-governance" docs\GOALS.md docs\ROADMAP.md docs\architecture\LIBRARY_PIPELINE.md docs\architecture\WORKSTREAM_LINKS.md docs\architecture\LANES.md`
  returned no matches.
- `git diff --check` passed.
- `python .\.trellis\scripts\task.py validate 06-05-06-05-provider-review-public-client-guards`
  passed.

## Technical Notes

- Implementation commit:
  `42869eaf test(api): guard provider governance public contracts`
- Archive commit:
  `f0b72a4a chore(task): archive 06-05-provider-review-public-client-governance`
- Current stale locations found in:
  `docs/GOALS.md`, `docs/ROADMAP.md`,
  `docs/architecture/LIBRARY_PIPELINE.md`, and
  `docs/architecture/WORKSTREAM_LINKS.md`.
