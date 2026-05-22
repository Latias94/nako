# NFO Sidecar Backup Policy Design

Status: Completed
Last updated: 2026-05-17

## Problem

M48 made the final write boundary safer by using local atomic replace, but it
does not keep the previous sidecar content anywhere. For self-hosted media
libraries, NFO files often include hand-authored fields or fields from another
media server. Even with preservation-aware rendering, a bug, unsupported NFO
shape, or user mistake should leave a clear recovery path.

The next risk is not XML rendering or atomicity. It is overwrite policy: when
Nako changes an existing local sidecar, users need a deterministic backup
artifact and item-level diagnostics.

## Target State

- Forced export over an existing local NFO sidecar creates a same-directory
  backup before replacing the sidecar.
- Backup mechanics live in VFS/local storage, not in `nako-nfo`'s XML codec.
- NFO export reports backup creation in internal/test-visible summaries.
- Backup failures fail the item before final sidecar replacement.
- Fresh sidecar creation does not create a backup.
- Existing public API, OpenAPI, SDK, and database schema remain unchanged.

## In Scope

- `crates/nako-vfs/src/lib.rs`
- `crates/nako-vfs/src/local.rs`
- `crates/nako-vfs/src/cache.rs` only if pass-through is needed
- `crates/nako-nfo/src/export.rs`
- `crates/nako-nfo/src/summary.rs`
- focused `nako-vfs` and `nako-nfo` tests
- goal/workstream documentation

## Out Of Scope

- No soft-link or hard-link management.
- No public HTTP API, OpenAPI, SDK, or protocol changes.
- No database schema or repository trait changes.
- No remote/WebDAV write support.
- No broad Jellyfin, Kodi, Plex, or Emby compatibility matrix.
- No provider breadth, metadata merge-policy redesign, playback, or transcode
  work.

## Architecture Direction

Extend the storage write boundary with an optional backup policy rather than
making `nako-nfo` manipulate filesystem paths directly:

```text
StorageBackupMode:
  None
  ExistingFile

StorageWriteRequest:
  uri
  content
  mode
  backup

StorageWriteReport:
  uri
  mode
  atomic
  backup

StorageBackupReport:
  original_uri
  backup_uri
```

`StorageBackend::write_string` remains a direct compatibility method. The
default `StorageBackend::write` should fail explicit backup requests on
unsupported backends rather than silently losing the backup policy. Local
storage should create backups in the same directory using a deterministic,
collision-resistant sidecar name before atomic replace.

Candidate backup naming:

```text
demo.nfo.nako-backup-20260517T120000Z
```

The first implementation can use millisecond or nanosecond timestamp precision
to avoid collisions. Retention pruning can be a follow-up unless the first slice
needs it to keep tests deterministic.

## NFO Workflow Direction

NFO export should request a backup only when it has confirmed an existing NFO
sidecar will be overwritten. That means:

- `force = true` and `stat(nfo_uri)` succeeds: request backup.
- missing sidecar: no backup.
- `force = false` and sidecar exists: skip unchanged.
- parse or preservation failure before write: no backup.
- backup failure: item fails before final write.

The export summary can gain an internal/test-visible backup count and per-item
backup report. If this turns out to leak into public DTOs, keep wire changes for
a later explicit API goal.

## Assumptions

| Assumption | Confidence | Evidence | Consequence if wrong |
| --- | --- | --- | --- |
| Backup should be same-directory for local sidecars. | High | Keeps backup near the file users edit and avoids cross-volume semantics. | If users need a central backup root, add it as a later configurable policy. |
| Backups should be explicit write policy, not codec behavior. | High | M47/M48 already separate XML ownership from persistence mechanics. | If codec starts creating backups, storage behavior becomes backend-specific inside XML code. |
| Public API exposure should wait. | High | M49 is a safety primitive; API design needs admin/client diagnostics review. | If UI needs this immediately, open a follow-up API/admin goal. |

## Closeout Condition

This lane can close when:

- local storage can create same-directory sidecar backups before atomic replace;
- unsupported backup requests fail explicitly;
- NFO forced export over an existing sidecar requests and reports a backup;
- fresh export does not create a backup;
- backup failure prevents final replacement;
- focused and workspace validation gates pass.
