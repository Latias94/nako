# NFO Storage Write Policy Design

Status: Active
Last updated: 2026-05-17

## Problem

M47 protects existing NFO XML fields during forced export, but the write path
still uses `StorageBackend::write_string`. For local files that means a direct
write to the final sidecar path. A crash, process abort, or IO failure can leave
a partial file. The workflow also reports export failures as strings, which
makes it hard to distinguish parse failures, preservation failures, conflicts,
and storage write failures.

The next risk is no longer XML ownership; it is persistence safety and
diagnostic clarity at the storage boundary.

## Target State

- Local NFO sidecar writes use an atomic temp-file-and-rename path where the
  backend supports it.
- VFS/storage owns write mechanics; NFO codec stays focused on XML.
- NFO export can distinguish parse, preservation, conflict, and write-stage
  failures in internal/test-visible diagnostics.
- Existing public API, OpenAPI, SDK, and database schema remain unchanged.
- Non-local or unsupported backends keep clear unsupported behavior rather than
  pretending to offer atomic writes.

## In Scope

- `crates/taru-vfs/src/lib.rs`
- `crates/taru-vfs/src/local.rs`
- `crates/taru-vfs/src/cache.rs` only if pass-through is needed
- `crates/taru-nfo/src/export.rs`
- `crates/taru-nfo/src/summary.rs`
- focused `taru-vfs` and `taru-nfo` tests
- goal/workstream documentation

## Out Of Scope

- No soft-link or hard-link management.
- No backup retention policy unless needed as a minimal diagnostic placeholder.
- No broad Jellyfin, Kodi, Plex, or Emby compatibility.
- No public HTTP API, OpenAPI, SDK, or protocol change.
- No database schema or repository trait changes.
- No provider breadth or metadata merge-policy redesign.
- No playback or transcode work.
- No new storage backend.

## Architecture Direction

Add a storage-owned write option rather than embedding file mechanics in
`taru-nfo`:

```text
StorageWriteMode:
  Direct
  AtomicReplace

StorageWriteRequest:
  uri
  content
  mode

StorageWriteReport:
  uri
  mode
  atomic
```

`StorageBackend::write_string` remains as the simple direct-write compatibility
method. A new default method can delegate to `write_string` for `Direct` and
return `Unsupported` for `AtomicReplace`. `LocalFsBackend` should override it
and implement `AtomicReplace` as:

1. Resolve and validate final writable path under the backend root.
2. Create a uniquely named temp file in the same parent directory.
3. Write full content.
4. Sync file data where practical.
5. Rename temp file to the final sidecar path.
6. Best-effort remove temp file on failure.

The NFO export workflow should request `AtomicReplace` for sidecar writes when
the backend advertises local writable semantics. If a backend cannot support
the requested mode, the item should fail with a clear diagnostic rather than
silently using a less safe mode.

## Diagnostics Direction

Keep diagnostics internal/test-visible in this slice. `NfoFailure` can gain a
classification enum if it does not change public API contracts yet, because NFO
job summary persistence is internal server data. If this turns out to leak into
public DTOs, keep the wire shape stable and add classification only inside
`taru-nfo` tests.

Candidate categories:

- `nfo_parse`
- `nfo_preservation`
- `nfo_conflict`
- `storage_write`
- `storage_unsupported`
- `missing_media_item`
- `unsupported_media_kind`

## Assumptions

| Assumption | Confidence | Evidence | Consequence if wrong |
| --- | --- | --- | --- |
| Local sidecar writes are the first backend worth making atomic. | High | WebDAV remains read-only for writes, and current M48 goal excludes new backends. | If remote write support arrives first, add backend-specific atomic semantics in that workstream. |
| Temp file and rename in the same directory is enough for the first safe local write boundary. | High | Standard filesystem practice; avoids cross-device rename. | If Windows rename semantics require remove/replace nuance, cover it in `LocalFsBackend` tests. |
| Internal NFO diagnostics can evolve without public API churn. | Medium | `NfoExportSummary` is used in app command output and job summaries, but public DTO exposure should be checked before implementation. | If public wire DTOs expose it, keep existing fields stable and add diagnostics in a follow-up API goal. |
| Backup/retention policy is separate from atomic write. | High | Backup changes user-visible storage behavior and needs its own options. | If atomic replace is not enough, open a follow-on backup policy goal. |

## Closeout Condition

This lane can close when:

- local atomic sidecar writes are implemented and tested through VFS;
- NFO export uses the safer write path for local sidecars;
- unsupported atomic write behavior is explicit for non-supporting backends;
- export failures carry test-visible diagnostic categories;
- M47 XML preservation behavior still passes;
- focused and workspace validation gates pass.
