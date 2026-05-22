# NFO Link Authority Design

Status: Complete
Last updated: 2026-05-21

## Why This Lane Exists

Nako now has explainable provider matching and strong NFO write primitives, but
local library authority is still incomplete. A real self-hosted library needs
safe answers for:

- whether NFO import/export is local authority or merely a suggestion;
- whether two **Media Sources** are duplicates because of local filesystem link
  evidence;
- whether Nako may create soft links or hard links later;
- how to prove a future link/write operation before it mutates library files.

The post-RPD umbrella deliberately ranked this lane before
`managed-import-staging`: downloads/import promotion must not write into a
library or create links until local authority and rollback semantics exist.

## Relevant Authority

- `CONTEXT.md`
- `docs/adr/0002-internal-vfs-before-os-mounting.md`
- `docs/adr/0007-metadata-merge-policy-and-local-authority.md`
- `docs/adr/0008-nfo-as-local-metadata-boundary.md`
- `docs/adr/0021-video-first-media-server-domain-model.md`
- `docs/workstreams/nfo-round-trip-preservation`
- `docs/workstreams/nfo-storage-write-policy`
- `docs/workstreams/nfo-sidecar-backup-policy`
- `docs/workstreams/nfo-backup-retention-diagnostics`

## Current Baseline

- `nako-nfo` preserves unknown NFO XML fields during forced export.
- `nako-vfs` supports local atomic replace, existing-file backup, and bounded
  backup retention.
- `LocalFsBackend` already exposes `LINKABLE` capabilities and detects
  symlink objects, but it does not expose link planning or mutation APIs.
- `SourceDuplicateRelationship` exists in `nako-core` and `nako-db`, with
  `FilesystemLink` evidence, but no product workflow currently creates link
  evidence from VFS diagnostics.

## Shipped Target State

- Link behavior starts with a non-destructive VFS dry-run contract.
- The dry-run explains whether a source URI, target URI, link kind, and backend
  are eligible before any filesystem mutation.
- Unsupported or remote backends report unsupported link planning explicitly.
- NFO sidecar authority remains separate from link mechanics.
- Source duplicate/link evidence flows through `SourceDuplicateRelationship`;
  no automatic item/source merge occurs.
- NFO import/export authority preview explains create, skip, forced update,
  backup-required, policy rejection, and failure decisions without writing
  sidecars or committing metadata.
- Actual link creation remains deferred until apply, backup/rollback, and
  audit reports are designed and tested.

## Completed Scope

- `nako-vfs` link planning types and local backend dry-run implementation.
- Non-mutating tests that prove dry-run does not create targets.
- Workstream documentation and post-RPD umbrella routing.
- Source Duplicate Relationship filesystem-link suggestions without automatic
  Media Source or Media Item merge.
- NFO authority preview for import/export sidecar decisions before mutation.

## Out Of Scope

- No symlink or hardlink apply operation in the first slice.
- No delete/cleanup/repair of links.
- No managed import/download promotion.
- No public client API contract.
- No broad NFO codec compatibility expansion.
- No direct Addon Sidecar filesystem access.

## Architecture Direction

### Boundary Split

```text
nako-nfo
  Owns XML parsing, NFO Round Trip, import/export summaries.

nako-vfs
  Owns backend capability, write mechanics, backup policy, and link planning.

nako-core / nako-db
  Own Source Duplicate Relationship persistence.

nako-server
  Later owns operator-facing diagnostics and acceptance workflows.
```

The first implementation slice must not make `nako-nfo` manipulate filesystem
paths or make `nako-server` infer local link behavior by inspecting OS paths.
Storage link semantics belong in `nako-vfs`.

### Link Planning Contract

Add a dry-run-only VFS model:

```text
StorageLinkKind:
  hard
  soft

StorageLinkPlanRequest:
  source_uri
  target_uri
  kind

StorageLinkPlanStatus:
  ready
  unsupported
  source_missing
  source_not_file
  target_parent_missing
  target_parent_not_directory
  target_exists
  security_violation

StorageLinkPlan:
  source_uri
  target_uri
  kind
  status
  can_apply
  source
  target
  message
```

The important property is non-mutation. A ready plan means "the backend can
prepare a future apply operation from this evidence", not "Nako has already
created a link".

### Link Evidence Flow

The implemented diagnostic workflow is:

1. Operator or import plan asks Nako to inspect link/duplicate candidates.
2. VFS returns link/inventory diagnostics.
3. Nako records suggested `SourceDuplicateRelationship` rows with
   `FilesystemLink` evidence.
4. Source and item identities remain unchanged.

### Link Apply Split Decision

Actual hardlink/symlink creation is deliberately split out of this lane.
The correct boundary is a follow-on lane after `managed-import-staging` opens,
because link application is not just a VFS operation:

- it must be tied to import/download promotion state;
- it needs rollback and cleanup semantics;
- it needs an audit report that can be redacted for Admin UI and webhook use;
- it must decide whether confirmed duplicate evidence is enough to mutate a
  library or whether operator approval is required;
- it must coordinate with backups and source duplicate acceptance.

This lane therefore ships **planning and authority diagnostics only**. Future
apply should be designed as `link-apply-and-import-promotion` or inside
`managed-import-staging`, not added ad hoc to `nako-vfs`.

## Closeout Condition

This lane can close when:

- VFS exposes non-destructive link planning with local and unsupported backend
  coverage;
- link evidence can be surfaced as Source Duplicate Relationship diagnostics
  without merging sources/items;
- NFO export/import authority has an explicit preview or diagnostic boundary
  consistent with existing NFO preservation/write/backup policies;
- no automatic link mutation exists before apply/rollback rules are proven;
- focused Rust gates, `cargo fmt --all -- --check`, and `git diff --check`
  pass.

All closeout conditions are satisfied as of 2026-05-21.
