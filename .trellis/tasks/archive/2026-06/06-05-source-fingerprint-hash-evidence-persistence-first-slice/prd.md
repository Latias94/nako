# Source fingerprint hash evidence persistence first slice

## Goal

Persist the redacted Source Fingerprint evidence produced by completed source
fingerprint hash jobs back onto the existing Media Source and Source State
records. This turns the current scheduler-executed hash report into durable
source identity evidence without adding public/operator surfaces or automatic
duplicate reconciliation.

## What I Already Know

- Source fingerprint hash execution, queued durable jobs, job summary JSON, and
  scheduler integration are already shipped.
- The hash executor returns `SourceFingerprintHashReport` with
  `SourceFingerprintEvidence`; the durable job summary intentionally omits raw
  fingerprint/hash material.
- `MediaSource.fingerprint` and `SourceState.fingerprint` already store
  redacted source identity evidence produced during scan ingestion.
- `SourceState` is the current candidate discovery surface used by scan
  reconciliation before creating `SourceDuplicateRelationship` suggestions.
- `SourceDuplicateRepository` exists, but there is no dedicated
  "find matching fingerprint candidates" repository method outside the scan
  ingestion planner.
- Architecture maps list evidence persistence as the next follow-on after
  scheduler integration, while Admin/Public API triggering and automatic
  reconciliation remain separate follow-ons.

## Requirements

- On successful source fingerprint hash execution, persist the report's
  redacted `SourceFingerprintEvidence.fingerprint` onto the current
  `MediaSource.fingerprint`.
- If an existing `SourceState` is present for the same library and current
  Source Locator, update only its `fingerprint`, preserving source id, scan id,
  tombstone state, size, etag, and modified-at facts.
- Reuse existing repository contracts where possible; do not add a schema
  migration unless implementation proves the existing records cannot represent
  the evidence.
- Keep durable job `summary_json` redaction-safe and continue excluding raw
  Source Locators, `StorageUri`, local paths, backend URLs, etags, credentials,
  raw digests, raw content hashes, and job input JSON.
- Preserve the existing job lease/runtime behavior: a failed evidence
  persistence step must fail the job rather than silently reporting success.
- Add focused tests for manual executor and scheduler-originated execution.

## Acceptance Criteria

- [ ] A successful full source fingerprint hash job persists a redacted
  fingerprint onto the Media Source.
- [ ] If the source has an existing Source State, the same redacted fingerprint
  is persisted onto that Source State without changing unrelated state fields.
- [ ] Scheduler-originated execution persists the same evidence and still marks
  the job succeeded.
- [ ] The job summary JSON remains redaction-safe and does not include the
  persisted fingerprint value, raw hash material, locator, or file contents.
- [ ] No Admin/Public API route, operator UI, duplicate merge, automatic Media
  Source merge, or new source-hash-specific runtime loop is added.
- [ ] Focused `nako-server` source hash tests pass under nextest.
- [ ] `cargo check -p nako-server --tests` passes.

## Out Of Scope

- No Admin/Public API route or DTO.
- No operator UI or manual trigger endpoint.
- No automatic duplicate relationship creation in this first slice.
- No automatic Media Source merge or item hierarchy mutation.
- No schema migration unless strictly required by the implementation.
- No durable evidence table separate from existing Media Source / Source State
  records.
- No broad scan ingestion rewrite.

## Technical Notes

- Likely files:
  - `crates/nako-server/src/app/source_hash.rs`
  - `crates/nako-server/src/app/tests/source_hash.rs`
  - Possibly `crates/nako-core/src/repository/media.rs`
  - Possibly `crates/nako-core/src/repository/scan.rs`
- Existing source hash execution currently returns only
  `SourceFingerprintHashJobSummary`; evidence persistence likely requires the
  internal execution path to keep the full `SourceFingerprintHashReport` long
  enough to persist its redacted evidence, then project to the summary.
- `MediaRepository::upsert_media_source` can update the current Media Source.
- `ScanRepository::get_source_state` and `ScanRepository::upsert_source_state`
  can update an existing Source State by current locator without inventing a
  scan snapshot.
- Persisted fingerprint values are redacted fingerprints already produced by
  `SourceFingerprintEvidence::from_scan_metadata`; raw VFS hash digests must
  not be persisted through this task.
- Specs/docs to keep in view:
  - `CONTEXT.md`
  - `docs/architecture/CONTROL_PLANE.md`
  - `docs/architecture/STORAGE_VFS.md`
  - `docs/architecture/LIBRARY_PIPELINE.md`
  - `.trellis/spec/nako-core/backend/*`
  - `.trellis/spec/nako-db/backend/*`
  - `.trellis/spec/nako-library/backend/*`
  - `.trellis/spec/nako-server/backend/*`

## Implementation Plan

1. Inspect the existing source hash execution return types and adjust the
   private execution path to retain `SourceFingerprintHashReport`.
2. Add a small persistence helper that updates `MediaSource.fingerprint` and an
   existing matching `SourceState.fingerprint`.
3. Keep public/internal command output summary-only so existing diagnostic
   redaction behavior does not widen.
4. Extend focused source hash tests to assert Media Source / Source State
   persistence and summary redaction for manual and scheduler paths.
5. Run focused nextest and `cargo check -p nako-server --tests`.
