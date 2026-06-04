# Source Fingerprint Escalation Map Reconciliation

## Goal

Reconcile active storage architecture maps after
`200015bf feat(library): add source fingerprint escalation policy` shipped the
typed source fingerprint escalation policy seam and
`.trellis/tasks/archive/2026-06/06-04-06-04-source-fingerprint-escalation-policy-first-slice/`
was marked completed.

## Requirements

- Remove `proposed:source-fingerprint-escalation-policy` from active proposed
  follow-on queues.
- Mark the completed escalation policy seam as shipped evidence where storage
  follow-ons are summarized.
- Preserve true remaining follow-ons:
  - `proposed:source-fingerprint-hash-execution`;
  - operator queueing and diagnostics for ambiguous source identity;
  - cache repair, watcher intake stability, and PostgreSQL runtime harness
    work.
- Do not change Rust code, storage behavior, scan behavior, hashing behavior,
  generated SDKs, or specs.

## Acceptance Criteria

- [x] Focused grep finds no `proposed:source-fingerprint-escalation-policy` in
      active docs.
- [x] Active docs reference `200015bf` or the archived task as shipped
      evidence.
- [x] Remaining source fingerprint follow-on stays visible and narrowly named
      as hash execution / operator diagnostics, not the completed policy seam.
- [x] `git diff --check` and Trellis task validation pass.

## Definition Of Done

- Docs-only diff.
- Task evidence records the grep/validation commands.
- Task is committed and archived.

## Technical Notes

- Implementation commit:
  `200015bf feat(library): add source fingerprint escalation policy`
- Completed task:
  `.trellis/tasks/archive/2026-06/06-04-06-04-source-fingerprint-escalation-policy-first-slice/`
- Current stale locations found in:
  `docs/architecture/STORAGE_VFS.md`,
  `docs/architecture/WORKSTREAM_LINKS.md`, and
  `docs/architecture/LANES.md`.
- `docs/architecture/STORAGE_VFS.md` already names
  `proposed:source-fingerprint-hash-execution` as the remaining execution
  follow-on.
