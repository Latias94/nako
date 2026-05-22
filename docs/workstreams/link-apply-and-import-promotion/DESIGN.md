# Link Apply And Import Promotion Design

Status: Complete
Last updated: 2026-05-21

## Why This Lane Exists

`managed-import-staging` now gives Nako a safe staged artifact model and a
non-mutating promotion preview. `nfo-link-authority` gives Nako VFS link
dry-run diagnostics and non-mutating NFO authority preview. The next product
risk is the first real apply: turning a staged artifact into a Media Source in a
Media Library.

This is a different risk class from preview. Preview explains intent; apply can
create files, hardlinks, symlinks, catalog rows, duplicate relationships, audit
records, and later sidecar writes. If any step fails halfway, Nako must not leave
an untracked file in a library root or a promoted Media Source that points to a
missing or unauthorized target.

## Relevant Authority

- `CONTEXT.md`
- `docs/adr/0002-internal-vfs-before-os-mounting.md`
- `docs/adr/0007-metadata-merge-policy-and-local-authority.md`
- `docs/adr/0008-nfo-as-local-metadata-boundary.md`
- `docs/adr/0014-durable-event-outbox-for-webhooks-and-automation.md`
- `docs/adr/0015-capability-scoped-http-addons-and-automation-providers.md`
- `docs/adr/0021-video-first-media-server-domain-model.md`
- `docs/workstreams/managed-import-staging`
- `docs/workstreams/nfo-link-authority`
- `docs/workstreams/addon-library-file-write-policy`

## Starting Baseline

- Managed Import artifacts are durable and library-scoped.
- Promotion preview computes destination, duplicate/link, NFO, provider, and
  blocked-reason diagnostics without mutation.
- VFS link planning exists as dry-run only.
- Server app service can expose Managed Import diagnostics internally.
- There is no durable promotion acceptance/apply record.
- There is no VFS-backed copy/link apply path tied to Managed Import.
- There is no rollback/cleanup record for partial promotion failure.

## Shipped Scope

As of closeout, this lane shipped:

- durable Managed Import promotion acceptance/apply/audit records with
  SQLite/PostgreSQL parity;
- explicit app-service acceptance and idempotent replay before mutation;
- VFS-mediated copy/hardlink/symlink apply primitives and typed unsupported
  backend behavior;
- server-side promotion apply orchestration that revalidates plan facts and
  writes catalog state only after target creation;
- duplicate relationship persistence from accepted promotion evidence;
- VFS-mediated cleanup after catalog commit failure, with cleanup-complete or
  cleanup-pending audit outcomes;
- a split decision that routes NFO sidecar mutation to
  `nfo-sidecar-promotion-apply`.

## Target State

- A promotion apply command is explicit, idempotent, and operator-confirmed.
- Apply always revalidates or snapshots the current promotion plan before
  storage mutation.
- Copy/hardlink/symlink are mediated by storage/VFS backends; server code never
  manipulates OS paths directly.
- Catalog writes happen only after the target locator is created or verified.
- Durable audit records explain every attempt and outcome with redacted storage
  diagnostics.
- Partial failures transition to rollback-complete or cleanup-pending states
  rather than pretending promotion succeeded.
- Move/delete source behavior remains deferred until source-retention and
  rollback semantics are proven.

## In Scope

- Core promotion acceptance/apply/audit domain records and repository traits.
- SQLite/PostgreSQL migrations and backend-neutral contract tests.
- Storage/VFS apply primitives needed for copy/hardlink/symlink targets.
- Server app service for explicit apply commands and redacted apply outcomes.
- Focused tests proving idempotency, stale-plan rejection, catalog consistency,
  and cleanup-pending behavior after injected failures.
- Workstream closeout routing to downloader/watch-folder acquisition only after
  apply safety is proven.

## Out Of Scope

- Torrent/Usenet/download-client protocols.
- Watch-folder daemon runtime.
- Move/delete source apply in the first slice.
- NFO sidecar export/import mutation in this lane.
- Public Client API.
- Admin UI.
- AI or Addon autonomous apply.

## Architecture Direction

### Boundary Split

```text
nako-core
  Owns promotion acceptance, apply request, operation kind, audit state, and
  repository traits.

nako-db
  Owns durable apply/audit schema and backend-neutral contract tests.

nako-vfs
  Owns storage mutation primitives for copy/link targets and any future cleanup
  primitive. It must preserve path-safety and backend capability checks.

nako-server
  Owns operator-facing app service orchestration: load artifact, preview,
  revalidate, apply storage operation, commit catalog/source state, and record
  audit/cleanup outcome.

nako-nfo
  Remains explicit authority for sidecar import/export. It is not called by the
  promotion apply path. Accepted NFO sidecar mutation is split to the dedicated
  `nfo-sidecar-promotion-apply` lane.
```

### Promotion Apply State Model

```text
requested -> validating -> applying_storage -> committing_catalog -> promoted
       \-> rejected
        \-> failed_before_mutation
          \-> cleanup_pending -> cleanup_complete
          \-> rollback_complete
```

A terminal `promoted` state means both storage and catalog state are consistent.
A storage-created target without catalog commit must not be called promoted.

### Operation Policy

First apply should support the lowest-risk operations first:

1. hardlink or symlink only when VFS planning returns ready and the backend can
   apply the selected link kind;
2. copy only when the storage backend can create a target without exposing raw
   OS paths to server code;
3. move/delete source remains deferred because it can destroy the staged
   artifact and complicates rollback.

### Idempotency

Every apply request needs an idempotency key scoped to artifact, selected
operation, destination locator, and accepted plan facts. Replaying the same key
must return the previous terminal or in-flight outcome rather than repeat a file
mutation. Replaying with a different operation or target must be rejected.

### Plan Revalidation

Apply must not trust an old preview DTO. Before mutation, it must check:

- artifact state allows promotion;
- source artifact URI still exists and matches expected facts when available;
- target locator remains inside the target library boundary;
- selected operation is still eligible;
- duplicate/provider/NFO warnings are either accepted or still block apply;
- target does not already exist unless the policy explicitly accepts reuse.

### Rollback And Cleanup

The apply orchestration should prefer compensating cleanup after a target was
created but catalog commit failed. If cleanup cannot be completed safely, record
cleanup-pending with redacted evidence and never mark the artifact promoted.
`rollback_complete` remains reserved for future restore/move/delete semantics;
this lane closes with cleanup-complete and cleanup-pending coverage for created
promotion targets.

### Audit And Redaction

Audit records must avoid raw source URLs, raw local paths, fingerprints, or raw
provider payloads in operator-facing reports. Internal records may keep stable
IDs and source locator schemes needed for diagnosis, following existing Nako
redaction conventions.

## NFO Sidecar Mutation Split Decision

LAIP-070 splits NFO sidecar import/export mutation to
`docs/workstreams/nfo-sidecar-promotion-apply`.

The reason is architectural boundary, not schedule deferral. Promotion apply
turns a staged artifact into a durable **Media Source** by creating or verifying
the target locator, then committing catalog state. NFO sidecar mutation is a
separate **Library File Write** and metadata-authority operation:

- export writes canonical metadata back to an NFO sidecar while preserving
  third-party XML through **NFO Round Trip**;
- import reads local NFO metadata into canonical metadata and may confirm a
  **Provisional Hierarchy**;
- both directions must respect field locks, local authority, per-library export
  policy, backup requirements, bounded backup retention, and redacted audit
  reporting;
- partial failure semantics are different from Media Source promotion because a
  sidecar write can succeed while metadata/audit commit fails, or metadata
  import can commit while no sidecar write occurred.

Therefore this lane owns only:

- accepted Managed Import promotion;
- VFS-mediated copy/hardlink/symlink target creation;
- Media Item / Media Source / Library Item State commits after target
  durability;
- duplicate relationship persistence tied to promotion evidence;
- cleanup-complete or cleanup-pending audit after promotion partial failure.

The follow-on NFO lane owns:

- explicit sidecar apply acceptance and idempotent replay;
- revalidation of the current NFO authority preview before mutation;
- backup, atomic replace, retention diagnostics, and rollback/repair-pending
  outcomes for sidecar writes;
- application of local NFO authority to canonical metadata, field locks, and
  hierarchy confirmation;
- redacted audit evidence that never exposes raw library paths.

LAIP must not call `nako-nfo` to mutate sidecars, and NFO sidecar apply must not
be smuggled into Managed Import promotion as an implicit post-hook.

## First Slice Recommendation

Start with durable acceptance/audit records before storage mutation:

1. Add promotion apply IDs, operation enums, state enums, accepted plan snapshot,
   and audit outcome model.
2. Add SQLite/PostgreSQL migrations and backend-neutral contract tests.
3. Add app service method that records/replays an explicit apply request but
   stops before storage mutation behind a `validated`/`accepted` state.
4. Then add VFS mutation primitives and first hardlink/symlink apply tests.

## Closeout Condition

This lane is closed because:

- [x] durable promotion acceptance/audit records exist with backend parity;
- [x] apply commands are explicit, idempotent, and revalidate preview facts;
- [x] copy/hardlink/symlink apply is mediated by VFS/storage, not OS paths;
- [x] catalog writes are committed only after target locator durability is proven;
- [x] cleanup-complete or cleanup-pending behavior is tested after injected
  partial failure;
- [x] NFO sidecar mutation is split to a dedicated accepted Library File Write
  lane;
- [x] focused Rust gates, `cargo fmt --all -- --check`, and `git diff --check`
  pass.
