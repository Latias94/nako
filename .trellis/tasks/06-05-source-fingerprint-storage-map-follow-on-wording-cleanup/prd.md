# Source Fingerprint Storage Map Follow-on Wording Cleanup

## Goal

Remove the last stale `hash scheduling / operator diagnostics` wording from
`docs/architecture/STORAGE_VFS.md` after the scheduling diagnostic planner and
architecture map reconciliation shipped.

## Requirements

- Update `docs/architecture/STORAGE_VFS.md` remote-storage follow-on wording to
  point to source fingerprint hash queue/operator integration rather than the
  completed scheduling diagnostics first slice.
- Keep source fingerprint status aligned with `LANES.md` and
  `WORKSTREAM_LINKS.md`.
- Do not change code, API, schema, queue behavior, or task scope beyond this
  docs-only correction.

## Acceptance Criteria

- [ ] The old `source fingerprint hash scheduling / operator diagnostics`
      phrase no longer appears in the target architecture maps.
- [ ] Queue/operator integration wording remains present where future work is
      described.
- [ ] `git diff --check` and Trellis context validation pass.

## Definition Of Done

- Docs correction is committed.
- Task evidence records verification commands.
- Task is archived.
