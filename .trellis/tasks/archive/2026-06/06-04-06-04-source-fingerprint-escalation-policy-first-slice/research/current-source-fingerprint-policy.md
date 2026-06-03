# Current source fingerprint policy research

- Query: choose the smallest useful source fingerprint escalation policy slice.
- Scope: `nako-core` domain policy plus `nako-library` source commit plan.
- Date: 2026-06-04.

## Findings

### Current code shape

- `crates/nako-core/src/media/source.rs` defines
  `SourceFingerprintEvidenceKind`, `SourceFingerprintPolicyInput`, and
  `SourceFingerprintEvidence`.
- `SourceFingerprintEvidence::from_scan_metadata` already creates redacted
  fingerprints from:
  - content-looking backend hashes at confidence 1000;
  - size + etag at confidence 800;
  - backend fingerprint at confidence 700;
  - size + modified time at confidence 500;
  - locator-only with no fingerprint.
- Stale evidence reduces confidence by 200.
- `can_preserve_source_identity` currently requires non-stale evidence,
  confidence >= 900, and a fingerprint.
- `can_suggest_duplicate` currently requires confidence >= 500 and a
  fingerprint.
- `crates/nako-library/src/ingestion/source_commit.rs` uses this evidence to:
  - find reconciliation candidates when a locator is new;
  - preserve source identity only for strong evidence and one eligible
    relocation candidate;
  - create suggested duplicate relationships for weak evidence rather than
    auto-merging.

### Architecture constraints

- `docs/architecture/STORAGE_VFS.md` names
  `proposed:source-fingerprint-escalation-policy` as an opt-in partial/full hash
  escalation follow-on.
- The same document warns that hashing entire multi-gigabyte files during scan
  can hurt NAS and cloud-backed libraries.
- `docs/architecture/LIBRARY_PIPELINE.md` treats source fingerprint evidence as
  layered evidence and duplicate suggestions, not mandatory full-file hashing.
- Library quality guidelines explicitly say to use source fingerprint evidence
  as duplicate evidence, not source identity.

## Recommended first slice

Add a pure typed policy decision:

- no escalation when an existing locator is being updated, evidence is already
  strong, or no ambiguous candidate exists;
- partial hash when one weak non-stale candidate needs confirmation;
- full hash when multiple weak candidates need disambiguation or stale
  ambiguous evidence needs refresh.

Expose that decision on `SourceObservationPersistencePlan` so future scan
diagnostics, operator queues, or hash jobs can consume it. Do not execute hash
work or persist new fields in this slice.

## Risks

- If the policy directly triggers hash reads, the task crosses into VFS/runtime
  work and can create expensive scan behavior.
- If weak evidence starts preserving source identity, it violates the library
  quality rule and can merge distinct media sources incorrectly.
- If raw etags, source locators, paths, or fingerprints are surfaced as reasons,
  the decision becomes unsafe for diagnostics.

## Verification candidates

- Core unit tests for each action/reason.
- Library ingestion test that a weak duplicate candidate still produces a
  duplicate suggestion, while the plan records partial-hash escalation.
- Library ingestion test that multiple candidates produce full-hash escalation
  without changing inserted disposition or duplicate suggestion count.
