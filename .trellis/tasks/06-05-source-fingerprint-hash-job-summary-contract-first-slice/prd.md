# Source fingerprint hash job summary contract first slice

## Goal

Define a redaction-safe durable job summary contract for future source
fingerprint hash lease execution. The summary should let a future executor write
operator-useful `summary_json` without exposing raw source locators, storage
URIs, backend URLs, credentials, etags, fingerprints, or hash material.

## What I Already Know

- `nako-library::source_hash` already returns `SourceFingerprintHashReport`
  from partial/full VFS-backed hash execution.
- The report contains redaction-safe evidence, but future durable job summaries
  should be even narrower and should not include the evidence fingerprint field.
- `nako-server` now has an internal planner that prepares in-memory
  `SourceFingerprintHashRequest` values for future execution.
- This task should not add any server executor, scheduler, API route, schema
  migration, or persistence behavior.

## Requirements

- Add a serializable `SourceFingerprintHashJobSummary` contract in
  `nako-library::source_hash`.
- The summary must include:
  - selected `SourceFingerprintHashMode`;
  - `SourceFingerprintEvidenceKind`;
  - confidence and stale state;
  - `bytes_hashed`.
- The summary must not include:
  - `SourceFingerprintEvidence::fingerprint`;
  - `StorageUri`, source locator, path, backend URL, etag, credential, raw digest,
    or input JSON content.
- Provide a conversion from `SourceFingerprintHashReport` into the summary.
- Add focused tests proving serialization is narrow and redaction-safe for both
  partial and full reports.

## Acceptance Criteria

- `cargo check -p nako-library --tests` passes.
- `cargo nextest run -p nako-library source_hash_job_summary --no-fail-fast`
  passes.
- Existing `source_hash` tests remain green.
- Architecture/spec docs describe the shipped summary contract and keep
  executor/scheduler/API/evidence persistence as follow-ons.

## Out Of Scope

- No durable job execution, scheduler loop, server runtime worker, or lease
  claim.
- No VFS read behavior changes.
- No Admin/Public API or DTO.
- No schema migration or evidence persistence.
- No duplicate relationship mutation or automatic Media Source merge behavior.

## Technical Notes

- Main code file: `crates/nako-library/src/source_hash.rs`.
- Likely docs/spec updates:
  - `.trellis/spec/nako-library/backend/quality-guidelines.md`
  - `docs/architecture/STORAGE_VFS.md`
  - `docs/architecture/CONTROL_PLANE.md`
  - `docs/architecture/LIBRARY_PIPELINE.md`
  - `docs/architecture/LANES.md`
