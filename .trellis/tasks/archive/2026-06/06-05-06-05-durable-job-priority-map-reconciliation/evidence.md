# Evidence

## 2026-06-05

- Task opened after active docs still listed the completed
  `proposed:durable-job-priority-policy-and-scheduler-migration` label while
  `CONTROL_PLANE.md` and `LANES.md` already described the generic priority
  baseline as shipped.
- Updated active docs to link the completed priority baseline task and
  `c1608abc`, then renamed the remaining follow-on to
  `proposed:durable-job-kind-scheduler-migration`.
- `rg -n "proposed:durable-job-priority-policy-and-scheduler-migration" docs\GOALS.md docs\ROADMAP.md docs\architecture\WORKSTREAM_LINKS.md docs\architecture\LANES.md docs\architecture\CONTROL_PLANE.md`
  returned no matches.
- `git diff --check` passed.
- `python .\.trellis\scripts\task.py validate .trellis\tasks\06-05-06-05-durable-job-priority-map-reconciliation`
  passed.
- Spec update review: no `.trellis/spec/` update needed because this docs-only
  reconciliation did not introduce or change executable API, command, database,
  infra, or cross-layer contracts.
