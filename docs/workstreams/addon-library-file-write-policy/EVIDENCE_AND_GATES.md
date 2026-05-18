# Addon Library File Write Policy Evidence And Gates

Status: Active
Last updated: 2026-05-18

## Smallest Current Repro

```powershell
rg -n "Library File Write|subtitle|NFO|nfo|StorageWriteRequest|StorageWriteReport|StorageBackupPolicy|atomic_replace|backup|sidecar" crates docs
git diff --check
```

This proves the file-write inventory is fresh before subtitle, NFO, or sidecar
write behavior is added.

## Gate Set

### Audit Gate

```powershell
rg -n "Library File Write|subtitle|NFO|nfo|StorageWriteRequest|StorageWriteReport|StorageBackupPolicy|atomic_replace|backup|sidecar" crates docs
git diff --check
```

### File Write Apply Gate

```powershell
cargo check -p taru-core -p taru-db -p taru-api -p taru-server -p taru-nfo -p taru-vfs --tests
cargo nextest run -p taru-server nfo --no-fail-fast
cargo nextest run -p taru-server addon_side_effect --no-fail-fast
cargo nextest run -p taru-server nfo --no-fail-fast
cargo nextest run -p taru-nfo --no-fail-fast
cargo nextest run -p taru-vfs --no-fail-fast
cargo fmt --all -- --check
git diff --check
```

ALFW-030 should add focused tests for MediaSource-targeted NFO export side
effects, redacted reports, and idempotent replay before broader package gates.

### Closeout Gate

```powershell
cargo fmt --all -- --check
git diff --check
```

Broaden to workspace checks if file-write behavior changes shared storage,
NFO, API, or repository boundaries.

## Evidence Anchors

- `docs/workstreams/addon-library-file-write-policy/DESIGN.md`
- `docs/workstreams/addon-library-file-write-policy/TODO.md`
- `docs/workstreams/addon-protected-writes/HANDOFF.md`
- `docs/workstreams/nfo-round-trip-preservation/`
- `docs/workstreams/nfo-storage-write-policy/`
- `docs/workstreams/nfo-sidecar-backup-policy/`
- `docs/workstreams/nfo-backup-retention-diagnostics/`
- `crates/taru-nfo/src/export.rs`
- `crates/taru-vfs/src/lib.rs`
- `crates/taru-vfs/src/local.rs`
- `crates/taru-server/src/app/nfo.rs`
- `crates/taru-server/src/app/addons.rs`

## Fresh Evidence

2026-05-18, ALFW-010:

- Workstream opened from APW-060 closeout as the follow-on for subtitle, NFO,
  and sidecar-asset Library File Write behavior.
- This is a planning split only; no file-write runtime behavior changed.
- Fresh validation remains required before marking ALFW-020 or later tasks
  complete.

2026-05-18, core-architecture-deepening CAD-070 alignment:

- At CAD-070 alignment time, ALFW remained proposed; no subtitle, NFO, or
  sidecar file-write runtime behavior existed yet.
- Added explicit guidance that NFO-derived metadata apply must reuse
  `MetadataRepository::commit_nfo_import` /
  `NfoImportPersistenceCommit`.
- Added explicit guidance that file-write paths which affect discoverable
  source state must reuse `ScanRepository::commit_library_scan_source`,
  `LibraryIndexRepository`, or a new first-party commit unit instead of
  Addon-specific source/state/search write ordering.

2026-05-18, ALFW-020 seam audit and first-target decision:

- Audit inputs:
  - `CONTEXT.md` confirms Library File Write is the Taru-owned boundary for
    subtitles, NFO files, artwork, and sidecar assets, and NFO Export is
    governed by local file-write policy.
  - `crates/taru-core/src/addon.rs` already has `library_file_write` and
    `subtitle_write` permissions, plus media item/source side-effect targets.
  - `crates/taru-server/src/app/addons.rs` currently applies only
    `metadata_write`; unsupported permissions are skipped with safe error
    codes.
  - `crates/taru-nfo/src/export.rs` derives NFO sidecar targets from
    `MediaSource`, preserves existing NFO XML on forced export, and writes via
    `StorageWriteRequest::atomic_replace`.
  - `crates/taru-vfs/src/lib.rs` and `crates/taru-vfs/src/local.rs` expose
    write modes, backup policy, write reports, same-directory atomic replace,
    existing-file backup, and keep-latest pruning.
  - `crates/taru-server/src/app/nfo.rs` already wraps NFO import/export in
    durable jobs, concurrency limiting, writable-backend checks, and redacted
    outbox event payloads.
- Decision: ALFW-030 should implement MediaSource-targeted addon-initiated
  Taru-owned NFO Export as the first Library File Write apply path. The addon
  may request an NFO export intent and write policy, but Taru derives the NFO
  sidecar URI and owns rendering, VFS write, backup, retention, and reporting.
- Target semantics:
  - `MediaSource` is the first supported target because it gives an
    unambiguous library root and sidecar derivation. `MediaItem` is deferred
    until multi-source/source-variant behavior is explicit.
  - The payload should be typed around NFO export intent, not raw XML. Safe
    policies are create-missing and replace-existing-preserving.
  - Create-missing skips existing sidecars; replace-existing-preserving must use
    NFO Round Trip rendering, atomic replace, existing-file backup, and backup
    retention.
  - The apply response/report must redact `StorageWriteReport`, `StorageUri`,
    Source Locators, filesystem paths, remote handles, backup URIs, and raw
    payload content.
- Core architecture alignment:
  - The selected first path exports Canonical Metadata to an NFO sidecar; it
    does not import addon-supplied NFO-derived Canonical Metadata. Any future
    NFO-derived metadata apply must route through `commit_nfo_import` /
    `NfoImportPersistenceCommit`.
  - The selected first path should not update Media Source, Source State, Local
    Inference Evidence, Library Item State, or search projection inline. If
    creating or replacing a sidecar later becomes discoverable source state, the
    update must route through `commit_library_scan_source`,
    `LibraryIndexRepository`, or a new first-party commit unit.
- Deferred alternatives:
  - Subtitle import is deferred because the codebase does not yet have a
    first-party subtitle/track import model, target semantics, or safe report
    boundary comparable to NFO export.
  - Arbitrary sidecar asset write is deferred because content-type validation,
    target derivation, backup policy, and redaction matrix are broader than the
    first NFO export slice.
- ALFW-030 warning: current Addon Side Effect apply status has
  `pending`/`applied`/`failed`/`skipped` only. If NFO export is queued as a job,
  ALFW-030 must add truthful queued/job association semantics or a redacted
  apply-report boundary before reporting completion.
- Validation:
  - `rg -n "Library File Write|subtitle|NFO|nfo|StorageWriteRequest|StorageWriteReport|StorageBackupPolicy|atomic_replace|backup|sidecar" crates docs`
    completed successfully; output was redirected to a temp file for review and
    contained 2374 inventory lines.
  - `Get-Content -Raw docs\workstreams\addon-library-file-write-policy\WORKSTREAM.json | ConvertFrom-Json | Out-Null`
    passed.
  - `git diff --check` passed with only Git CRLF normalization warnings for
    the edited workstream docs.
