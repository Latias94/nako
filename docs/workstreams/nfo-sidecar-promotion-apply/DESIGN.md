# NFO Sidecar Promotion Apply Design

Status: Complete
Last updated: 2026-05-21

## Why This Lane Exists

`nfo-link-authority` shipped non-mutating NFO authority preview, and the NFO
safety lanes shipped preservation-aware export, VFS-backed atomic writes,
same-directory backup, bounded backup retention, and diagnostics.
`link-apply-and-import-promotion` shipped accepted Managed Import promotion and
deliberately split NFO sidecar mutation out at LAIP-070.

The remaining risk is actual sidecar mutation. A real self-hosted library needs
NFO import/export to be useful, but sidecar writes and local-authority imports
are too dangerous to hide behind scan, refresh, provider matching, or Managed
Import promotion. They need an explicit accepted apply boundary.

## Relevant Authority

- `CONTEXT.md`
- `docs/adr/0002-internal-vfs-before-os-mounting.md`
- `docs/adr/0007-metadata-merge-policy-and-local-authority.md`
- `docs/adr/0008-nfo-as-local-metadata-boundary.md`
- `docs/adr/0014-durable-event-outbox-for-webhooks-and-automation.md`
- `docs/adr/0015-capability-scoped-http-addons-and-automation-providers.md`
- `docs/adr/0021-video-first-media-server-domain-model.md`
- `docs/workstreams/nfo-round-trip-preservation`
- `docs/workstreams/nfo-storage-write-policy`
- `docs/workstreams/nfo-sidecar-backup-policy`
- `docs/workstreams/nfo-backup-retention-diagnostics`
- `docs/workstreams/nfo-link-authority`
- `docs/workstreams/link-apply-and-import-promotion`
- `docs/workstreams/addon-library-file-write-policy`

## Current Baseline

- `nako-nfo` owns NFO parsing/export and preservation-aware round-trip updates.
- VFS storage writes can perform local atomic replace and backup behavior.
- Backup retention diagnostics exist for NFO sidecar backups.
- NFO authority preview can explain create, skip, forced update,
  backup-required, policy rejection, and failure decisions without mutation.
- Managed Import promotion can create/copy/link a Media Source target and commit
  catalog state with cleanup audit, but it does not mutate NFO sidecars.

## Problem

Without this lane, future code could accidentally perform sidecar writes in the
wrong place:

- a metadata refresh could export NFO as a hidden post-hook;
- Managed Import promotion could write sidecars after catalog commit without a
  separate acceptance record;
- an Addon or automation flow could bypass Nako-owned file-write policy;
- NFO import could overwrite canonical metadata or field locks without a
  durable authority explanation.

These are different invariants from promotion apply. Sidecar apply must protect
both library files and canonical metadata authority.

## Target State

- NFO sidecar apply is explicit, operator- or policy-accepted, idempotent, and
  auditable.
- Apply always revalidates the current NFO authority preview before mutation.
- NFO export uses **NFO Round Trip** and VFS-backed Library File Write behavior;
  server code never writes raw OS paths directly.
- NFO import applies local authority to canonical metadata, field locks, and
  **Hierarchy Confirmation** only through a recorded acceptance workflow.
- Backup, atomic replace, bounded retention, rollback, and repair-pending
  outcomes are visible in redacted audit records.
- Sidecar apply is reusable by future Admin API, automation, and Addon side
  effects without granting direct filesystem access.

## In Scope

- Core/domain model for sidecar apply IDs, operation kind, state, accepted
  preview snapshot, and audit outcomes.
- Repository traits and database adapters/migrations if durable replay cannot
  reuse an existing acceptance record safely.
- Server app-service acceptance and idempotent replay for NFO sidecar apply.
- Export apply: canonical metadata to NFO sidecar using round-trip preservation,
  backup, atomic replace, and retention diagnostics.
- Import apply: NFO sidecar to canonical metadata, local authority/field locks,
  and hierarchy confirmation.
- Partial-failure tests for backup/write/import/audit commit boundaries.
- Redaction rules for paths, XML payloads, provider payloads, and diagnostics.

## Out Of Scope

- Broad Jellyfin/Kodi NFO compatibility expansion.
- Managed Import media target promotion.
- Downloader, watch-folder, torrent, or Usenet acquisition.
- Public Client API or Admin UI design.
- Addon autonomous direct writes.
- Artwork/subtitle sidecar mutation, except where shared Library File Write
  policy is referenced.

## Architecture Direction

### Boundary Split

```text
nako-nfo
  Owns XML parsing, round-trip update planning, import/export summaries, and
  preservation/conflict reports.

nako-vfs
  Owns storage write mechanics: atomic replace, backup, backup retention,
  capability checks, and redacted storage diagnostics.

nako-core
  Owns sidecar apply command/state/audit records, local-authority effects, and
  repository traits.

nako-db
  Owns durable sidecar apply persistence and backend-neutral tests.

nako-server
  Owns acceptance orchestration: load current media item/source state,
  revalidate preview, apply NFO import/export through codec and VFS boundaries,
  commit metadata or audit state, and return redacted outcomes.
```

`nako-server` must not write sidecar files through raw OS APIs. `nako-nfo` must
not decide storage path permissions. VFS must not interpret canonical metadata
authority. Each layer owns one part of the safety boundary.

### Operation Model

First-class operation kinds:

- `export_sidecar`: canonical metadata -> NFO sidecar file;
- `import_sidecar`: NFO sidecar file -> canonical metadata/local authority;
- `round_trip_update`: update Nako-owned fields while preserving unknown XML,
  if this needs to be distinguished from a create-only export.

The operation kind must be part of the idempotency key scope together with media
library, media item/source identity, sidecar locator, accepted preview facts,
and policy version.

### State Model

```text
requested -> validating_preview -> accepted
accepted -> writing_sidecar -> committed
accepted -> importing_metadata -> committed
accepted -> rejected
accepted -> failed_before_mutation
writing_sidecar -> repair_pending
writing_sidecar -> rollback_complete
importing_metadata -> repair_pending
```

A terminal `committed` state means the accepted sidecar operation and the
corresponding audit/metadata state are consistent. If Nako cannot prove
consistency after a partial failure, it must record `repair_pending` instead of
claiming success.

### Preview Revalidation

Preview is explanation, not authorization. Before mutation, apply must re-check:

- media library policy still allows the operation;
- media item/source still matches the accepted target;
- sidecar locator remains within the library boundary;
- existing sidecar facts still match the accepted preview when present;
- backup-required, force-overwrite, and retention warnings are accepted;
- field locks and local authority policy are still compatible;
- hierarchy confirmation target still exists and is not structurally stale.

### Export Apply Semantics

Export apply writes NFO sidecars only through VFS storage APIs. It must:

- create or update only the accepted sidecar locator;
- preserve unknown and third-party XML through **NFO Round Trip**;
- create backup before forced overwrite when policy requires it;
- record backup locator IDs and retention diagnostics without raw OS paths;
- refuse hidden destructive rewrites unless the accepted preview explicitly
  allowed the policy.

### Import Apply Semantics

Import apply reads NFO through `nako-nfo` and commits local authority through
core/server repository boundaries. It must:

- distinguish local authority from provider suggestions;
- respect user-locked fields;
- record which fields were accepted, skipped, locked, or conflicted;
- confirm provisional hierarchy only when the accepted NFO evidence still
  matches the target;
- avoid rewriting sidecar files unless the operation is explicitly export or
  round-trip update.

### Rollback, Repair, And Audit

Sidecar apply cannot rely only on best-effort file cleanup:

- if export fails before mutation, record `failed_before_mutation`;
- if export writes but audit/metadata commit fails, record or recover to
  `repair_pending` with backup evidence;
- if rollback safely restores the previous sidecar from backup, record
  `rollback_complete`;
- if import commits metadata but follow-up audit fails, do not silently retry
  metadata mutation without idempotent replay rules;
- audit messages must redact raw paths, raw XML, provider payloads, and secrets.

## First Slice Recommendation

Start with durable sidecar apply acceptance and audit records before any file or
metadata mutation:

1. Add sidecar apply IDs, operation/state enums, accepted preview snapshot, and
   audit outcome model.
2. Add repository traits and backend-neutral tests.
3. Add server acceptance/replay method that stops before mutation.
4. Then implement export apply, because it exercises VFS write/backup behavior
   without first changing canonical metadata.
5. Implement import authority application after export failure semantics are
   proven.

## Closeout Condition

This lane can close when:

- sidecar apply commands are explicit, idempotent, and preview-revalidated;
- export writes use NFO Round Trip plus VFS-backed backup/atomic write/retention
  behavior;
- import applies local authority through canonical metadata, field-lock, and
  hierarchy-confirmation boundaries;
- partial failures produce rollback-complete, failed-before-mutation, or
  repair-pending audit states rather than silent success;
- no raw OS path, raw XML, or provider payload leaks into operator-facing
  diagnostics;
- focused Rust gates, `cargo fmt --all -- --check`, and `git diff --check`
  pass.

## Closeout Decision — 2026-05-21

This lane is closed because the target state is implemented and freshly
verified:

- NFO sidecar apply is explicit, operator-accepted, idempotent, and preview
  revalidated.
- Export writes go through `nako-nfo` round-trip preservation and VFS
  backup/atomic write/retention behavior.
- Import applies local authority through canonical metadata, field-lock, and
  hierarchy-confirmation boundaries.
- Partial failures produce `failed_before_mutation`, `rollback_complete`, or
  `repair_pending` terminal outcomes without false committed state.
- VFS exposes a restore boundary so backup-backed rollback is storage-owned,
  not a raw OS-path write from server orchestration.
- Operator-facing outcomes redact raw local paths, raw XML, provider payloads,
  and secrets.

Follow-on exposure is intentionally split:

- Admin API and UI can surface preview/accept/apply/replay diagnostics.
- Public Client API should not expose raw sidecar paths or direct file writes.
- Addons may request NFO side effects only through scoped Nako-owned apply
  commands.
- Downloads/watch-folder acquisition must produce staged artifacts and consume
  Managed Import promotion plus NFO sidecar apply; it must not bypass either
  boundary.
