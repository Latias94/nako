# Source fingerprint escalation policy first slice

## Goal

Add a bounded, typed source-fingerprint escalation decision so Nako can explain
when an ambiguous source-identity observation should be upgraded to partial or
full content hashing later, without executing expensive hashing or changing
source identity behavior in this slice.

## What I already know

* The long-horizon queue lists source fingerprint escalation as a storage/VFS
  follow-on.
* `docs/architecture/STORAGE_VFS.md` explicitly warns that full-file hashing can
  be too expensive and recommends layered evidence: size, mtime, path, duration,
  stream facts, partial hash, then full hash only when needed.
* `nako-core::SourceFingerprintEvidence` already classifies scan metadata into
  `ContentHash`, `SizeAndEtag`, `SizeAndModifiedTime`,
  `BackendFingerprint`, and `LocatorOnly`, with confidence and redacted
  fingerprint values.
* `nako-library::ingestion::source_commit` currently uses fingerprint evidence
  for duplicate suggestions and strong relocation, but it has no typed way to
  say whether a weak/ambiguous case should escalate to hashing.
* Existing library quality guidance says source fingerprint evidence must remain
  duplicate evidence, not automatic source identity.

## Assumptions

* This first slice should be read-only planning/diagnostic state, not hash
  execution.
* The policy belongs in `nako-core` because evidence strength and safe reasons
  are domain-level concepts.
* `nako-library` should attach the policy decision to the in-memory
  `SourceObservationPersistencePlan` so future scan/operator surfaces can use
  it without changing schema now.

## Requirements

* Add a typed source fingerprint escalation decision in `nako-core`.
* Express at least:
  * no escalation;
  * partial hash recommended;
  * full hash recommended.
* Express safe reasons such as existing locator, strong evidence, no ambiguous
  candidate, confirm single weak candidate, disambiguate multiple candidates,
  and refresh stale ambiguous evidence.
* Keep all reasons and values redaction-safe; do not expose raw locators,
  etags, fingerprints, paths, or backend URLs.
* Add the decision to `nako-library` source observation persistence planning.
* Preserve current source commit behavior:
  * strong content hash can still preserve source identity;
  * weak evidence still creates duplicate suggestions instead of automatic
    source merge;
  * no schema, repository, API, Admin, VFS, or hashing-runtime changes.
* Add focused core and library tests for policy decisions and source commit plan
  exposure.

## Acceptance Criteria

* [x] Core tests cover no escalation for existing locator / strong evidence /
  no ambiguous candidate.
* [x] Core tests cover partial-hash recommendation for one weak candidate.
* [x] Core tests cover full-hash recommendation for multiple candidates and
  stale ambiguous evidence.
* [x] Library ingestion plan exposes the escalation decision without changing
  disposition, source IDs, or duplicate relationship creation.
* [x] `cargo fmt --all -- --check`, `cargo check -p nako-core -p nako-library --tests`,
  focused `cargo nextest`, `git diff --check`, and Trellis validate pass.

## Definition of Done

* Code and tests are committed with a Conventional Commit message.
* Verification evidence is persisted in this task directory.
* Reusable policy conventions are written back to relevant specs or
  architecture docs.
* Task is archived and the developer journal is recorded.

## Out of Scope

* No partial-hash or full-file hash execution.
* No scan scheduling, background job, durable queue, or VFS read changes.
* No database schema, repository trait, migration, API, Admin contract, or
  generated TypeScript change.
* No automatic merge based on weak source fingerprint evidence.
* No raw fingerprint, etag, path, source locator, backend URL, or provider
  payload exposure.

## Technical Approach

* Add core types in `crates/nako-core/src/media/source.rs`, likely:
  * `SourceFingerprintEscalationAction`;
  * `SourceFingerprintEscalationReason`;
  * `SourceFingerprintEscalationDecision`;
  * a policy input or method on `SourceFingerprintEvidence`.
* Use candidate count and existing-locator state as policy inputs. Keep the
  output small, serializable, and redaction-safe.
* Add a `fingerprint_escalation` field to
  `nako-library::ingestion::SourceObservationPersistencePlan`.
* Compute the decision in `plan_source_observation_commit` after reconciliation
  candidates are known.
* Add tests next to existing core fingerprint policy tests and existing source
  commit tests.

## Research References

* [`research/current-source-fingerprint-policy.md`](research/current-source-fingerprint-policy.md)
  - current code shape, policy gap, and bounded first-slice recommendation.

## Technical Notes

* Relevant specs:
  * `.trellis/spec/nako-core/backend/index.md`
  * `.trellis/spec/nako-core/backend/quality-guidelines.md`
  * `.trellis/spec/nako-library/backend/index.md`
  * `.trellis/spec/nako-library/backend/quality-guidelines.md`
  * `.trellis/spec/guides/cross-layer-thinking-guide.md`
* Relevant docs:
  * `docs/architecture/STORAGE_VFS.md`
  * `docs/architecture/LIBRARY_PIPELINE.md`
* Likely write scope:
  * `crates/nako-core/src/media/source.rs`
  * `crates/nako-library/src/ingestion/source_commit.rs`
  * `.trellis/spec/nako-core/backend/quality-guidelines.md` or
    `.trellis/spec/nako-library/backend/quality-guidelines.md`
  * `docs/architecture/STORAGE_VFS.md` and/or
    `docs/architecture/LIBRARY_PIPELINE.md`
