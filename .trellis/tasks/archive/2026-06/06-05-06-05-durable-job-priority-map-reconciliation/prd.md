# Durable Job Priority Scheduler Map Reconciliation

## Goal

Reconcile active roadmap and architecture maps after
`c1608abc feat(control-plane): add durable job priority policy` shipped the
generic durable-job priority baseline and
`.trellis/tasks/archive/2026-06/06-02-01f-durable-job-priority-policy-and-scheduler-migration/`
was marked completed.

## Requirements

- Remove `proposed:durable-job-priority-policy-and-scheduler-migration` from
  active proposed follow-on queues.
- Mark the shipped generic priority policy baseline as completed evidence where
  control-plane follow-ons are summarized.
- Preserve true remaining follow-ons:
  - broader job-kind scheduler migration onto typed budget-admitted scheduler
    paths;
  - control-plane observability / trace context;
  - API scale/cache contracts;
  - remote access and addon manager lifecycle work.
- Do not change Rust code, migrations, runtime behavior, generated SDKs, or
  specs.

## Acceptance Criteria

- [x] Focused grep finds no
      `proposed:durable-job-priority-policy-and-scheduler-migration` in active
      docs.
- [x] Active docs reference `c1608abc` or the archived task as shipped
      evidence.
- [x] Remaining scheduler migration follow-on stays visible and narrowly named
      as broader job-kind scheduler migration, not as the completed priority
      baseline.
- [x] `git diff --check` and Trellis task validation pass.

## Definition Of Done

- Docs-only diff.
- Task evidence records the grep/validation commands.
- Task is committed and archived.

## Technical Notes

- Implementation commit:
  `c1608abc feat(control-plane): add durable job priority policy`
- Completed task:
  `.trellis/tasks/archive/2026-06/06-02-01f-durable-job-priority-policy-and-scheduler-migration/`
- Current stale locations found in:
  `docs/GOALS.md` and `docs/architecture/WORKSTREAM_LINKS.md`.
- Adjacent broad wording normalized in:
  `docs/ROADMAP.md` and `docs/architecture/LANES.md`.
- `docs/architecture/CONTROL_PLANE.md` already distinguishes the shipped
  generic priority policy from broader job-kind scheduler migration.
