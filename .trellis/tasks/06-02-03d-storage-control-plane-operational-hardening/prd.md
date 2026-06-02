# Storage Control Plane Operational Hardening

## Goal

Advance storage/VFS and control-plane operational readiness with one bounded
slice that improves large-library reliability, playback input pressure, scan
scheduling, source fingerprint escalation, or diagnostics.

## Requirements

- Audit current storage health, VFS cache/staging diagnostics, library scan,
  and control-plane surfaces before selecting the exact slice.
- Prefer a narrow operational slice with measurable diagnostics or scheduling
  behavior.
- Keep Source Locator, Source Fingerprint, and Storage Backend Health language
  aligned with `CONTEXT.md`.
- Keep storage errors redaction-safe; do not expose local paths, signed URLs,
  credentials, or raw backend blobs in API/Web surfaces.
- Coordinate with playback runtime lane before changing playback source-read or
  artifact pressure semantics.
- Add focused tests for the selected storage/control-plane behavior.

## Acceptance Criteria

- [ ] The worker selects one bounded operational hardening slice after audit.
- [ ] Storage or scan pressure behavior is observable through typed diagnostics,
  tests, or control-plane state.
- [ ] Existing VFS/local/WebDAV behavior remains compatible.
- [ ] Redaction behavior is covered where diagnostics/API/Web output changes.
- [ ] Follow-ons are recorded for source fingerprint escalation, scan
  scheduling, PostgreSQL harness, or playback artifact pressure if not selected.

## Definition of Done

- Focused VFS/library/server/db tests pass for changed behavior.
- API/Web/diagnostic tests pass if surfaces change.
- Evidence notes record selected slice, commands, and deferred follow-ons.

## Out of Scope

- No broad storage rewrite.
- No new deployable service.
- No playback runtime contract change without planner coordination.
- No raw path or credential exposure.

## Technical Notes

- Candidate slices from lane docs: source fingerprint escalation, playback
  artifact/source-read pressure, scan scheduling, PostgreSQL runtime harness.
- Likely files: `crates/nako-vfs/src/**`, `crates/nako-library/src/scan.rs`,
  `crates/nako-server/src/app/storage.rs`, `crates/nako-server/src/app/staging.rs`,
  and storage diagnostics routes.
- Stop for planner coordination if the selected slice needs schema changes
  outside storage-owned tables or shared playback response changes.
