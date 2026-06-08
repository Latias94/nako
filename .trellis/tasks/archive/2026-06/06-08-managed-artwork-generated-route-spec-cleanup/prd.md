# Managed artwork generated route spec cleanup

## Goal

Remove stale Managed Artwork Admin route exclusion wording left after
`process-next` was generated. The code baseline now has zero explicit Admin
route exclusions, so specs must not tell future agents that `process-next`
remains excluded.

## Requirements

- Update `.trellis/spec/nako-api/backend/quality-guidelines.md` so the requeue
  scenario says `process-next` is generated separately.
- Remove the stale Bad-case wording that implies generating `process-next` is
  wrong.
- Keep the zero-exclusion baseline documented.
- No production code changes.

## Acceptance Criteria

- [x] `rg` finds no stale `process-next` exclusion language in the Managed
  Artwork scenarios.
- [x] `python ./.trellis/scripts/task.py validate .trellis/tasks/06-08-managed-artwork-generated-route-spec-cleanup` passes.
- [x] `git diff --check` passes.

## Definition of Done

- Spec, task evidence, and validation are updated together.
- Commit with a Conventional Commit message, then archive the task in a
  separate chore commit.
