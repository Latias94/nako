# Admin Settings API Restoration Map Reconciliation

## Goal

Reconcile active roadmap and architecture maps after
`.trellis/tasks/archive/2026-06/06-02-01b-admin-settings-api-backed-restoration/`
was marked completed and the Admin settings API/Web restoration surface shipped.

## Requirements

- Remove `proposed:admin-settings-api-backed-restoration` from active proposed
  follow-on queues.
- Mark the completed Admin Settings API Backed Restoration task as shipped
  evidence where Web Product, Operations/Release, and generated-artifact
  follow-ons are summarized.
- Preserve true remaining follow-ons:
  - `proposed:config-hot-apply-and-restart-required-model`;
  - Web public client browse/release smoke/player UX follow-ons;
  - operations remote access, backup classification, and config restart policy
    work.
- Do not change Rust code, Admin Web code, generated SDKs, runtime behavior, or
  specs.

## Acceptance Criteria

- [x] Focused grep finds no `proposed:admin-settings-api-backed-restoration`
      in active docs.
- [x] Active docs reference the archived task as shipped evidence.
- [x] Remaining config mutation follow-on stays visible and narrowly named as
      hot-apply/restart-required behavior, not API-backed restoration.
- [x] `git diff --check` and Trellis task validation pass.

## Definition Of Done

- Docs-only diff.
- Task evidence records the grep/validation commands.
- Task is committed and archived.

## Technical Notes

- Completed task:
  `.trellis/tasks/archive/2026-06/06-02-01b-admin-settings-api-backed-restoration/`
- Current stale locations found in:
  `docs/ROADMAP.md` and `docs/architecture/WORKSTREAM_LINKS.md`.
- `docs/architecture/LANES.md` already states Web Product is idle after Admin
  settings API restoration.
- `docs/architecture/OPERATIONS_RELEASE.md` already preserves config
  hot-apply/restart-required model as the remaining settings follow-on.
