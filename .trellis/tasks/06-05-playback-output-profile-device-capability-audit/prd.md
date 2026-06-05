# Playback output profile and device capability audit

## Goal

Decide the next stable playback output and device capability contract before
implementing more decoding/transcode features such as HEVC/AV1 output,
hardware tone mapping, image subtitle burn-in, TV profiles, mobile/native
profiles, or richer renderer behavior.

## Requirements

- Audit current `nako-playback`, `nako-transcode`, server playback flow, and
  Public/Admin DTO boundaries.
- Decide which facts belong to Client Applications, output profiles, Admin
  diagnostics, and operator policy.
- Identify the first safe executable follow-on after the audit.
- Keep this task architecture-only unless explicitly converted later.
- Preserve the separation between Playback Source Selection, Playback Runtime,
  Transcode Pipeline planning, and FFmpeg command execution.

## Acceptance Criteria

- [ ] The audit defines the minimum output/device capability facts needed for
      the next playback/transcode wave.
- [ ] HEVC/AV1, HDR tone mapping, subtitle burn-in, and device profile
      follow-ons are ranked with prerequisites.
- [ ] Public Client API versus Admin diagnostics responsibilities are
      documented.
- [ ] Unsafe parallel combinations are listed.
- [ ] The audit recommends one first executable playback/transcode task.

## Definition of Done

- Research output is written under this task or linked from the parent audit.
- Any architecture-map update is explicit and limited.
- No Rust/TypeScript implementation changes unless the task is later expanded.
- `git diff --check` passes.

## Out of Scope

- No HEVC/AV1 execution.
- No hardware tone-map execution.
- No subtitle burn-in implementation.
- No Public Client or Admin DTO changes in this audit task.

## Technical Notes

- Parent audit: `.trellis/tasks/06-05-06-05-cross-lane-architecture-audit/`
- Key research:
  - `.trellis/tasks/06-05-06-05-cross-lane-architecture-audit/research/playback-decoding-transcode.md`
  - `.trellis/tasks/06-05-06-05-cross-lane-architecture-audit/research/synthesis.md`
- Important docs/ADRs:
  - `docs/architecture/PLAYBACK.md`
  - ADR 0038, 0044, 0045, 0049, 0052, 0053
