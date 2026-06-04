# Library Scan Scheduling And Storage Admission

## Goal

Improve large-library reliability by adding a bounded scan scheduling or
storage-health admission slice that prevents scan/probe work from blindly
pressuring unhealthy storage or staging resources.

## Requirements

- Audit library scan orchestration, VFS/storage backend health, staging
  diagnostics, durable job scheduling, and probe/intake workflow before
  selecting the exact slice.
- Prefer a small, observable admission policy for scan/probe work over a broad
  scan rewrite.
- Use Nako terms: Storage Backend Health, Storage Circuit Breaker, Source
  Locator, Source Fingerprint, and Media Source.
- Keep diagnostics redaction-safe; do not expose local paths, credentials,
  Source Locators, Source Fingerprints, signed URLs, or raw backend errors.
- Add focused tests for scheduling/admission behavior and compatibility with
  existing local/WebDAV behavior.

## Acceptance Criteria

- [ ] The selected scheduling/admission slice is documented with the reason it
  is the smallest useful product-readiness step.
- [ ] Scan/probe admission behavior reacts to storage health or staging pressure
  through typed policy, diagnostics, or durable job state.
- [ ] Existing VFS local/WebDAV behavior remains compatible.
- [ ] Redaction behavior is covered if API or diagnostics output changes.
- [ ] Follow-ons are recorded for source fingerprint escalation, file watcher,
  PostgreSQL harness, or broader scheduler priority work if not selected.

## Definition of Done

- Focused VFS/library/server/db tests pass for changed behavior.
- API/Admin/Web checks pass if diagnostic surfaces change.
- `cargo fmt --all -- --check` passes.
- Evidence notes record selected slice, commands, and deferred follow-ons.

## Out of Scope

- No broad storage rewrite.
- No new deployable scheduler service.
- No full file watcher productization unless the audit proves it is the minimal
  selected slice.
- No schema migration without planner approval.
- No raw path, Source Locator, Source Fingerprint, or credential exposure.

## Technical Notes

- Likely areas: `crates/nako-library`, `crates/nako-vfs`,
  `crates/nako-server/src/app/storage.rs`, durable job surfaces, and storage
  diagnostics routes.
- Coordinate with playback if admission affects source-read or staging policy.
