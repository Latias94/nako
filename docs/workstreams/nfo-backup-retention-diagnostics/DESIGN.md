# NFO Backup Retention And Diagnostics Design

Status: Completed
Last updated: 2026-05-17

## Problem

M49 creates rollback artifacts before overwriting existing local NFO sidecars,
but the backup set is unbounded. A busy library or repeated forced exports can
accumulate backup files indefinitely. The NFO export summary records creation
and failure, but it does not yet explain retention pruning decisions.

The next risk is operational: users need predictable disk growth and admins
need a way to inspect what happened during backup creation and pruning.

## Target State

- Local NFO backups have an explicit bounded retention policy.
- Retention pruning happens in storage/local policy, not in the XML codec.
- NFO export summaries expose internal/test-visible counts and reports for
  backup creation, pruning, and pruning failures.
- Admin-facing diagnostics can inspect backup/pruning results without adding
  fields to public client protocol crates.
- Public client API, OpenAPI public routes, SDK inventory, database schema, and
  media-provider behavior remain unchanged unless a real gap is discovered.

## In Scope

- `crates/nako-vfs/src/lib.rs`
- `crates/nako-vfs/src/local.rs`
- `crates/nako-vfs/src/cache.rs` only if pass-through is needed
- `crates/nako-nfo/src/export.rs`
- `crates/nako-nfo/src/summary.rs`
- `crates/nako-nfo/src/lib.rs` tests
- `crates/nako-api` and `crates/nako-server` only if admin diagnostics need a
  route/DTO adapter update
- focused and workspace validation
- goal/workstream documentation

## Out Of Scope

- No soft-link or hard-link management.
- No broad Jellyfin, Kodi, Plex, or Emby compatibility matrix.
- No provider breadth, metadata merge-policy redesign, playback, or transcode
  work.
- No new storage backend.
- No public client protocol changes.
- No database schema changes unless the implementation proves volatile
  summaries are insufficient.

## Architecture Direction

Extend the storage-owned backup policy with retention rather than making
`nako-nfo` enumerate and delete filesystem paths directly:

```text
StorageBackupPolicy:
  mode
  retention

StorageBackupRetention:
  keep_latest: Option<usize>

StorageBackupReport:
  original_uri
  backup_uri
  pruned_backups
  prune_failures
```

`LocalFsBackend` should identify Nako-created backups for a sidecar by the M49
name prefix:

```text
demo.nfo.nako-backup-*
```

It should sort backup candidates deterministically by file name or modified
time, keep the newest configured count, and remove older Nako backup files. The
first implementation should be conservative: it only prunes files matching the
Nako backup naming pattern for the same sidecar.

## NFO Workflow Direction

NFO export should request retention when it requests a backup. The first
default can be small and explicit in `nako-nfo`, for example keep the latest 5
backups per sidecar. Future config can move this into library options, but M50
should avoid schema/config churn unless needed.

Summary diagnostics should answer:

- how many sidecars were backed up;
- which backup URI was created for each source;
- how many old backups were pruned;
- which prune failures occurred, if any;
- whether a backup/pruning failure prevented sidecar replacement.

## Admin Diagnostics Direction

Prefer reusing job summary/internal command output first. Only add admin API
DTO/route changes if current admin surfaces cannot inspect NFO job summary
details. Do not add fields to `nako-client-protocol` or public client route
inventory in this slice.

## Assumptions

| Assumption | Confidence | Evidence | Consequence if wrong |
| --- | --- | --- | --- |
| Volatile job summaries are enough for first admin diagnostics. | Medium | NFO job summaries already include export/import details. | If admins need historical queries beyond job summaries, open a DB schema goal. |
| Retention belongs in storage policy. | High | M48/M49 already keep XML codec separate from file mechanics. | If NFO workflow starts deleting files directly, backend behavior will leak upward. |
| Keep-latest count is the right first retention policy. | High | Simple, deterministic, and avoids time/clock policy ambiguity. | Add age-based retention later if users need it. |

## Closeout Evidence

This lane closed after:

- local backup writes can prune older Nako backups with a bounded keep-latest
  policy;
- pruning never deletes non-matching files;
- pruning failures are reported in backup diagnostics;
- NFO forced export records backup and pruning diagnostics;
- public client protocol inventory remains unchanged;
- focused and workspace validation gates pass.

The first retention default is intentionally hard-coded in `nako-nfo` as a
small workflow policy. Configurable library/profile retention remains a
follow-on, not part of this slice.
