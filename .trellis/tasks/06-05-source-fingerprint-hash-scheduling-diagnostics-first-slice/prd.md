# Source Fingerprint Hash Scheduling Diagnostics First Slice

## Goal

Add a typed, redaction-safe scheduling/diagnostic planner for source fingerprint
hash escalation decisions so future scan or operator workflows can turn the
existing advisory `SourceFingerprintEscalationDecision` into an explicit
partial/full hash request without duplicating policy logic or exposing source
locators.

## Requirements

- Add a `nako-library` planning seam that accepts:
  - a `StorageUri` for the source to hash;
  - the existing `SourceFingerprintEscalationDecision`;
  - an opt-in scheduling policy with a configured partial-hash prefix length.
- The planner must produce a redaction-safe report for every input:
  - escalation action and reason;
  - evidence kind, confidence, stale state, and candidate count;
  - source scheme only, not the full source locator or local path;
  - whether hash execution should be scheduled;
  - if scheduled, the exact `SourceFingerprintHashMode` and
    `SourceFingerprintHashRequest`.
- `None` escalation and disabled scheduling must not schedule execution.
- `PartialHash` must schedule `SourceFingerprintHashMode::Partial` with the
  configured prefix length.
- `FullHash` must schedule `SourceFingerprintHashMode::Full`.
- A zero partial-hash prefix must fail with a redaction-safe
  `NakoError::InvalidInput` message.
- The slice must not execute VFS reads, enqueue durable jobs, persist evidence,
  change schema, add Admin/Public API fields, mutate duplicate relationships, or
  automatically merge Media Sources.
- Add focused `nako-library` tests for disabled, no-escalation, partial, full,
  invalid prefix, and locator/path redaction behavior.
- Update relevant library spec and storage/library architecture notes.

## Acceptance Criteria

- [ ] `nako-library` exposes a typed hash scheduling diagnostic planner.
- [ ] Planner output can be consumed by future queue/operator code without
      reading raw source locators from the diagnostic surface.
- [ ] Partial/full decisions produce the same explicit hash modes used by the
      existing hash execution kernel.
- [ ] Disabled/no-op decisions remain diagnostic-only.
- [ ] Focused `nako-library` tests and `cargo check` pass.
- [ ] Specs and architecture docs distinguish this planning seam from actual
      queue/API/execution integration.

## Definition Of Done

- Rust code and tests are committed.
- Task evidence records verification commands.
- Specs and architecture docs reflect the scheduling/diagnostics planning seam.
- Task is archived.

## Technical Notes

- Existing advisory policy:
  `SourceFingerprintEvidence::escalation_decision`.
- Existing execution kernel:
  `crates/nako-library/src/source_hash.rs`.
- Existing boundaries:
  `.trellis/spec/nako-library/backend/quality-guidelines.md`
  `docs/architecture/STORAGE_VFS.md`
  `docs/architecture/LIBRARY_PIPELINE.md`
- Likely write scope:
  - `crates/nako-library/src/source_hash.rs`
  - `.trellis/spec/nako-library/backend/quality-guidelines.md`
  - `docs/architecture/STORAGE_VFS.md`
  - `docs/architecture/LIBRARY_PIPELINE.md`
