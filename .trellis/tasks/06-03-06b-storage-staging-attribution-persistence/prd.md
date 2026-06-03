# Storage Staging Attribution Persistence

## Goal

Persist authoritative staging attribution so storage pressure, diagnostics, and
scan-admission behavior can represent ambiguous same-root or multi-endpoint
library staging honestly.

## Context

Wave 05 shipped scoped staging admission and queued scan fairness, but the
archive evidence still calls out persisted attribution for ambiguous same-root
multi-endpoint library staging records as a follow-on. This task owns that
storage/VFS authority improvement.

## Requirements

* Audit current staging manifests, storage pressure policy slices, scan
  admission reads, and Admin/server diagnostics before choosing the exact
  persistence seam.
* Persist only authoritative attribution facts on staging manifest records. If a
  staging record cannot be confidently tied to one library, model that ambiguity
  explicitly instead of deriving ownership from a source path prefix.
* Add a minimal durable attribution shape in `nako-core` and persist it through
  SQLite and PostgreSQL staging adapters with aligned migrations.
* Preserve SQLite and PostgreSQL parity for changed repository contracts.
* Keep scan-admission and diagnostics consumers honest: attributed, ambiguous,
  unknown, and multi-owner cases must not collapse into false per-library
  ownership.
* Server staging policy may count an attributed record toward the matching
  library slice and backend slice; ambiguous or unknown records count only
  toward backend pressure.
* Admin diagnostics may expose attribution kind and optional library id, but not
  raw source locators or storage-local details.
* Preserve redaction of raw source locators, local paths, source fingerprints,
  backend credentials, etags, headers, and host filesystem details.

## Out of Scope

* No watcher runtime, debounce, or filesystem event productization. That belongs
  to `06-03-06a-library-watcher-runtime-productization`.
* No Jellyfin reference research. That belongs to
  `06-03-06c-targeted-jellyfin-watcher-reference`.
* No broad PostgreSQL runtime suite expansion beyond gates required by changed
  attribution persistence.
* No cache repair operator workflow.
* No source fingerprint escalation policy.
* No new staging-pressure policy unless it is the minimal consumer needed to
  prove persisted attribution correctness.

## Acceptance Criteria

* [ ] Staging attribution has a persisted authority on manifest records.
* [ ] Ambiguous same-root and multi-endpoint cases remain explicit and do not
      create false per-library attribution.
* [ ] SQLite/PostgreSQL repository contract coverage exists for changed
      persistence behavior.
* [ ] Any server/Admin diagnostics remain redaction-safe.
* [ ] Follow-ons are recorded for repair workflows, broader PostgreSQL runtime
      suites, or source fingerprint escalation if discovered.

## Definition of Done

* Code changes are confined to the staging attribution contract, persistence
  adapters, server consumers, Admin DTOs/contracts, tests, and required specs.
* SQLite and PostgreSQL migration arrays register the same new version if a
  schema change is added.
* Focused contract and server/Admin tests prove attribution round-trips,
  ambiguous same-root records do not inflate per-library slices, and diagnostics
  remain redaction-safe.
* `cargo fmt --all -- --check` and `git diff --check` pass.

## Technical Approach

* Add a first-class `StagingAttribution` domain value in `nako-core` with three
  states: `attributed(library_id)`, `ambiguous`, and `unknown`.
* Store the attribution on `staging_manifest_records` using a kind column plus an
  optional library id column. Historical or fixture records should default to
  `unknown`.
* Update SQLite and PostgreSQL row mappers, insert/update SQL, and contract
  tests together so invalid persisted combinations fail as database errors.
* Change server staging-budget consumers to trust the persisted attribution
  field for library slices instead of matching `source_uri` path prefixes.
* Thread explicit attribution through known-library staging paths such as scan
  probe input and playback FFmpeg input; use `unknown` for hand-written or
  context-free fixtures.
* Extend Admin staging record DTOs and generated TypeScript contracts with
  attribution kind and optional library id while preserving existing redaction
  boundaries.

## Decision (ADR-lite)

**Context**: Wave 05 derived scoped staging pressure from `source_uri` and
configured roots. That improves backend-level pressure, but same-root WebDAV or
multi-endpoint libraries can still look like one library when path prefixes are
used as authority.

**Decision**: Persist a minimal authoritative attribution value directly on the
staging manifest record rather than creating a separate attribution table or
continuing to infer ownership from paths.

**Consequences**: Consumers gain a stable authority for per-library slices and
can represent ambiguous records honestly. The trade-off is a schema migration
and row-mapper expansion across SQLite/PostgreSQL, which must be covered by
contract tests and migration registration checks.

## Suggested Gates

* `cargo check -p nako-db -p nako-server -p nako-library --tests`
* Focused `cargo nextest run -p nako-db <staging-or-storage-filter> --no-fail-fast`
* Focused `cargo nextest run -p nako-server <staging-or-scan-filter> --no-fail-fast`
* PostgreSQL contract harness only if changed contracts require it
* `cargo fmt --all -- --check`
* `git diff --check`

## Coordination Notes

* Coordinate with 06a only if watcher runtime implementation needs the same
  attribution model or DTO in its first slice.
* Do not absorb the deferred broader PostgreSQL runtime suite lane into this
  task.

## Technical Notes

* `crates/nako-server/src/app/storage.rs` currently derives library slices from
  staging record source schemes and roots; this is the consumer that must stop
  inventing false ownership.
* `crates/nako-core/src/staging.rs`, `crates/nako-db/src/sqlite/staging.rs`, and
  `crates/nako-db/src/postgres/vfs_staging.rs` are the expected persistence
  touchpoints.
* Relevant evidence: `.trellis/tasks/archive/2026-06/06-03-05a-staging-budget-per-backend-policy/evidence.md`.
