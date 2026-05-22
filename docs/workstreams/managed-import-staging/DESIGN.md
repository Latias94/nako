# Managed Import Staging Design

Status: Complete
Last updated: 2026-05-21

## Why This Lane Exists

Nako now has explainable metadata matching and explicit local file authority:

- `metadata-provider-breadth` made provider capability, match ambiguity, and
  candidate review visible before canonical metadata writes.
- `nfo-link-authority` added VFS link dry-run, Source Duplicate Relationship
  filesystem-link suggestions, and non-mutating NFO authority preview.

That makes the next product risk clear: downloads, watch-folder imports, and
Addon-proposed artifacts must not write directly into a media library. They
need a Nako-owned staging and promotion boundary that can validate bytes,
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
- Existing VFS staging manifest implementation in `nako-core`, `nako-db`, and
  `nako-server/src/app/staging.rs`

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
- Import candidates are represented as Nako-owned staged artifacts, not direct
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

This target state is satisfied for staging and planning as of 2026-05-21.
Actual promotion apply is intentionally split to
`link-apply-and-import-promotion` because it is the first lane that may mutate a
Media Library root.

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
nako-core
  Owns Managed Import staging domain records, IDs, states, and repository traits.

nako-db
  Owns schema/migrations and repository adapters.

nako-vfs
  Owns storage staging, path safety, write/link planning primitives.

nako-metadata / nako-nfo / nako-catalog
  Provide diagnostics consumed by promotion planning; they do not own import
  lifecycle state.

nako-server
  Owns operator-facing app services, job/runtime orchestration, and Admin HTTP
  boundary later.
```

### Vocabulary

```text
Managed Import Source
  Operator URL, watched candidate, or Addon-proposed artifact descriptor. It is
  not a downloader implementation.

Managed Import Artifact
  Nako-owned staged file outside media library roots with lifecycle state,
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

The shipped Managed Import Staging lane stops at this safe planning boundary.
States after `planned` remain lifecycle vocabulary for the follow-on apply
lane, not behavior that staging silently performs.

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

## MIS-050 Apply Split Decision — 2026-05-21

Decision: split actual promotion apply into a dedicated follow-on workstream,
`link-apply-and-import-promotion`.

Rationale:

- Promotion apply crosses the boundary from explanation into file-system and
  library mutation.
- A preview can be recomputed and discarded safely; an apply can leave a Media
  Library root, Media Source row, NFO sidecar, duplicate relationship, and audit
  stream partially updated.
- Hardlink/symlink apply depends on VFS planning, but it is not just a VFS
  feature. It needs Managed Import artifact state, operator confirmation,
  duplicate/source semantics, rollback, cleanup, and redacted audit reporting.
- Copy/move/link strategy must be tied to a confirmed plan snapshot and
  revalidated at apply time. A stale preview is not an authorization to write.
- Move is particularly risky because it can remove the source artifact before
  catalog commit is proven. The follow-on should make move support explicit and
  may defer it behind copy/link apply.

Minimum apply requirements before any media-library mutation:

1. **Operator confirmation** — a command must select an operation and confirm
   the target library, destination locator, duplicate policy, and local/NFO
   authority implications.
2. **Plan revalidation** — apply must re-run or verify the preview facts and
   reject blocked or stale plans before touching storage.
3. **Durable audit** — every attempt needs an idempotency key, selected
   operation, state transitions, redacted diagnostics, and terminal outcome.
4. **VFS-only mutation** — server code must not manipulate OS paths directly;
   copy/move/link/delete/cleanup behavior must be mediated by storage
   backends.
5. **Rollback/cleanup** — if storage succeeds and database/catalog commit
   fails, Nako must either remove the target it created or record a durable
   cleanup-pending state with enough redacted evidence for an operator.
6. **Catalog consistency** — a Media Source should not appear as promoted until
   the target locator is durable and the repository transaction commits.
7. **NFO boundaries** — NFO import/export remains explicit. Promotion apply
   must not smuggle sidecar writes unless backup and authority rules are part
   of that accepted operation.

This lane therefore ships quarantine, diagnostics, durable artifact records,
and non-mutating promotion preview. The follow-on owns the first mutating apply
path.

## Closeout Condition

This lane can close when:

- Managed Import has durable staged artifact and promotion-plan records;
- app/service diagnostics are redacted and operator-oriented;
- non-mutating promotion planning consumes metadata/NFO/link evidence;
- first apply behavior is either proven with rollback/audit or explicitly split;
- focused Rust gates, `cargo fmt --all -- --check`, and `git diff --check`
  pass.

All Managed Import Staging closeout conditions are satisfied as of 2026-05-21.
The first apply behavior is explicitly split to
`docs/workstreams/link-apply-and-import-promotion`.
