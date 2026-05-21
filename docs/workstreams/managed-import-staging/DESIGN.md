# Managed Import Staging Design

Status: Active
Last updated: 2026-05-21

## Why This Lane Exists

Taru now has explainable metadata matching and explicit local file authority:

- `metadata-provider-breadth` made provider capability, match ambiguity, and
  candidate review visible before canonical metadata writes.
- `nfo-link-authority` added VFS link dry-run, Source Duplicate Relationship
  filesystem-link suggestions, and non-mutating NFO authority preview.

That makes the next product risk clear: downloads, watch-folder imports, and
Addon-proposed artifacts must not write directly into a media library. They
need a Taru-owned staging and promotion boundary that can validate bytes,
explain metadata identity, detect duplicates, plan NFO/link/file operations,
and produce an auditable promotion decision.

## Relevant Authority

- `CONTEXT.md`
- `docs/adr/0002-internal-vfs-before-os-mounting.md`
- `docs/adr/0007-metadata-merge-policy-and-local-authority.md`
- `docs/adr/0008-nfo-as-local-metadata-boundary.md`
- `docs/adr/0015-capability-scoped-http-addons-and-automation-providers.md`
- `docs/adr/0021-video-first-media-server-domain-model.md`
- `docs/workstreams/metadata-provider-breadth`
- `docs/workstreams/nfo-link-authority`
- Existing VFS staging manifest implementation in `taru-core`, `taru-db`, and
  `taru-server/src/app/staging.rs`

## Current Baseline

- VFS staging manifests already exist for probe and FFmpeg input lifecycles.
- Staging records track source URI, purpose, path, size, etag/fingerprint,
  state, leases, retention, and cleanup.
- Staging cleanup handles expired, failed, and pending reservations safely.
- Existing staging is runtime/cache oriented, not import-promotion oriented.
- There is no durable import proposal, import artifact, promotion plan, or
  operator acceptance workflow for writing into a Media Library.

## Target State

- Import staging has its own domain vocabulary and repository contract.
- Import candidates are represented as Taru-owned staged artifacts, not direct
  library writes.
- A staged artifact can be probed and matched without becoming a Media Source.
- Promotion planning produces explicit decisions:
  - destination library and target Source Locator;
  - copy/move/hardlink/symlink strategy eligibility;
  - duplicate relationship hints;
  - provider/NFO/local inference diagnostics;
  - required backups and rollback/audit implications;
  - reasons promotion is blocked.
- Promotion apply remains separate from the first planning slice unless apply,
  rollback, cleanup, and audit are fully proven.

## In Scope

- Planning docs and task ledger for managed import staging.
- Core domain model for import staged artifacts and promotion plan state.
- Repository contracts and SQLite/PostgreSQL migrations for the first durable
  import staging records.
- Server app service for creating/listing/diagnosing import staged artifacts.
- Non-mutating promotion plan preview.
- Validation gates that prove no library files are written during staging or
  preview.

## Out Of Scope

- Protocol-specific download clients.
- Automatic promotion apply in the first slice.
- Hardlink/symlink mutation apply before rollback/audit design.
- Public Client API.
- Addon manager/distribution.
- AI autonomous writes.

## Architecture Direction

### Boundary Split

```text
taru-core
  Owns Managed Import staging domain records, IDs, states, and repository traits.

taru-db
  Owns schema/migrations and repository adapters.

taru-vfs
  Owns storage staging, path safety, write/link planning primitives.

taru-metadata / taru-nfo / taru-catalog
  Provide diagnostics consumed by promotion planning; they do not own import
  lifecycle state.

taru-server
  Owns operator-facing app services, job/runtime orchestration, and Admin HTTP
  boundary later.
```

### Vocabulary

```text
Managed Import Source
  Operator URL, watched candidate, or Addon-proposed artifact descriptor. It is
  not a downloader implementation.

Managed Import Artifact
  Taru-owned staged file outside media library roots with lifecycle state,
  validation facts, and redacted diagnostics.

Managed Import Plan
  Non-mutating plan that explains how an artifact could become a Media Source,
  Source Variant, duplicate relationship, NFO sidecar, or blocked candidate.

Promotion Apply
  Future mutation step that writes/copies/moves/links into a Media Library only
  after explicit acceptance and rollback/audit readiness.
```

### State Machine Draft

```text
proposed -> staging -> staged -> inspected -> planned -> accepted -> applying -> promoted
                                  \-> rejected
                 \-> failed
promoted/rejected/failed -> cleanup_pending -> cleaned
```

The first executable slice should likely stop at `planned`, because apply has
higher data-loss risk and should be split unless fully proven.

### Existing Staging Reuse Decision

Do not overload existing `StagingManifestRecord` as the user-facing import
artifact model. It is valuable as a low-level VFS/cache lifecycle primitive, but
Managed Import needs product semantics: source kind, target library, operator
intent, inspection facts, promotion plan status, redacted diagnostics, and
acceptance state.

The likely shape is:

- keep VFS staging manifests as byte-storage/cache records;
- add Managed Import records that may reference a staging manifest;
- use promotion planning to bridge from staged artifact to future library write.

## First Slice Recommendation

Open with a schema/model slice, not a downloader:

1. Add core IDs, enums, records, and repository trait for Managed Import
   artifacts.
2. Add DB migrations and contract tests for SQLite/PostgreSQL parity.
3. Add app service methods to create manual staged-artifact records and list
   redacted diagnostics.
4. Do not stage external network bytes yet; seed staged artifacts from existing
   local/VFS staging evidence in tests.

## Closeout Condition

This lane can close when:

- Managed Import has durable staged artifact and promotion-plan records;
- app/service diagnostics are redacted and operator-oriented;
- non-mutating promotion planning consumes metadata/NFO/link evidence;
- first apply behavior is either proven with rollback/audit or explicitly split;
- focused Rust gates, `cargo fmt --all -- --check`, and `git diff --check`
  pass.