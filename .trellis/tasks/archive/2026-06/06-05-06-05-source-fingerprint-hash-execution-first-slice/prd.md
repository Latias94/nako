# Source Fingerprint Hash Execution First Slice

## Goal

Add a bounded source fingerprint hash execution kernel so future scan/operator
work can execute the existing source fingerprint escalation recommendation
without changing source identity behavior in this slice.

## Requirements

- Add a `nako-library` execution helper that computes source fingerprint hash
  evidence through the existing `StorageBackend` byte-read boundary.
- Support two explicit modes:
  - partial hash: reads a bounded prefix range and returns redaction-safe weak
    `BackendFingerprint` evidence;
  - full hash: streams the full object and returns redaction-safe strong
    `ContentHash` evidence.
- Keep outputs redaction-safe:
  - do not expose raw paths, locators, etags, backend URLs, or credentials;
  - do not expose the raw SHA-256 digest in the returned source evidence.
- Preserve current source identity behavior:
  - no automatic media source merge;
  - no duplicate relationship mutation;
  - no repository schema or migration;
  - no Admin/Public API or generated TypeScript contract.
- Add focused tests for partial range selection, full streaming, evidence kind,
  redaction, and unsupported backend/read failure propagation.
- Update relevant `.trellis/spec/` and architecture notes because this advances
  source fingerprint hash execution beyond the previous advisory-only seam.

## Acceptance Criteria

- [ ] Partial hash execution reads only the configured prefix range.
- [ ] Full hash execution streams bytes without using `read_range(None)`.
- [ ] Partial hash result returns `SourceFingerprintEvidenceKind::BackendFingerprint`
      and does not expose raw hash bytes.
- [ ] Full hash result returns `SourceFingerprintEvidenceKind::ContentHash` and
      does not expose raw hash bytes.
- [ ] Unsupported or failing backend reads propagate as `NakoError`.
- [ ] Focused `cargo nextest` / `cargo check`, `git diff --check`, and Trellis
      validation pass.

## Definition Of Done

- Rust code and tests are committed.
- Specs and architecture docs reflect the new execution-kernel boundary.
- Task evidence records verification commands.
- Task is archived.

## Technical Notes

- Existing advisory policy:
  `.trellis/tasks/archive/2026-06/06-04-06-04-source-fingerprint-escalation-policy-first-slice/`
- Current architecture follow-on:
  `docs/architecture/STORAGE_VFS.md` `proposed:source-fingerprint-hash-execution`
- Likely write scope:
  - `crates/nako-library/src/source_hash.rs`
  - `crates/nako-library/src/lib.rs`
  - `.trellis/spec/nako-library/backend/quality-guidelines.md`
  - `docs/architecture/STORAGE_VFS.md`
