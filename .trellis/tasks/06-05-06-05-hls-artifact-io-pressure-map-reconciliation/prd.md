# HLS Artifact IO Pressure Map Reconciliation

## Goal

Reconcile active playback/storage architecture maps after
`48668afc feat(playback): enforce hls artifact io admission` shipped HLS
artifact I/O pressure enforcement and
`.trellis/tasks/archive/2026-06/06-02-01d-hls-artifact-io-pressure-enforcement/`
was marked completed.

## Requirements

- Remove `proposed:hls-artifact-io-pressure-enforcement` from active proposed
  follow-on queues.
- Mark the completed HLS artifact I/O pressure task as shipped evidence where
  playback and storage follow-ons are summarized.
- Preserve true remaining follow-ons:
  - playback resource admission queueing / waitlist;
  - playback OS/device capacity tuning;
  - player recovery UX;
  - storage cache repair, source fingerprint execution/escalation, watcher
    intake stability, and PostgreSQL runtime harness work.
- Do not change Rust code, storage behavior, playback behavior, release scripts,
  generated SDKs, or specs.

## Acceptance Criteria

- [x] Focused grep finds no `proposed:hls-artifact-io-pressure-enforcement`
      in active docs.
- [x] Active docs reference `48668afc` or the archived task as shipped
      evidence.
- [x] Remaining playback/storage follow-ons stay visible and narrowly named.
- [x] `git diff --check` and Trellis task validation pass.

## Definition Of Done

- Docs-only diff.
- Task evidence records the grep/validation commands.
- Task is committed and archived.

## Technical Notes

- Implementation commit:
  `48668afc feat(playback): enforce hls artifact io admission`
- Completed task:
  `.trellis/tasks/archive/2026-06/06-02-01d-hls-artifact-io-pressure-enforcement/`
- Current stale locations found in:
  `docs/architecture/STORAGE_VFS.md` and
  `docs/architecture/WORKSTREAM_LINKS.md`.
- `docs/architecture/LANES.md` already states playback-transcode is idle after
  HLS artifact I/O pressure enforcement.
