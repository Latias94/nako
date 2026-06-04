# Current Map Drift

## Observation

The related hierarchy application path is now shipped in two focused tasks:

- backend-only safe application:
  `.trellis/tasks/archive/2026-06/06-02-01c-provider-review-related-hierarchy-application/`
- Admin-only read-only plan/apply surface:
  `.trellis/tasks/archive/2026-06/06-05-provider-hierarchy-application-admin/`

However, current maps still list
`proposed:provider-review-related-hierarchy-application` as pending in:

- `docs/GOALS.md`
- `docs/ROADMAP.md`
- `docs/architecture/LIBRARY_PIPELINE.md`
- `docs/architecture/WORKSTREAM_LINKS.md`

`docs/architecture/LANES.md` is already closer to the new truth: the
library-metadata-control-plane lane is idle after related hierarchy application
and durable batch execution follow-ons landed.

## Reconciliation Boundary

Update current maps only. Keep legacy workstream closeouts as historical
evidence because they describe the state at their own closeout dates.

Remaining true follow-ons should be narrower than the shipped lane:

- Douban TV/episode endpoint depth.
- Provider review Public Client governance.
- Mutation-capable provider governance undo with persisted rollback snapshot.
- Admin Web UX for related hierarchy plan/apply.
- Durable/bulk related hierarchy execution, if operators need it.

## Validation

- `python ./.trellis/scripts/task.py validate .trellis/tasks/06-05-architecture-related-hierarchy-closeout-reconciliation`
- `git diff --check`
