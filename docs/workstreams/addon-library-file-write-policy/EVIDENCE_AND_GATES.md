# Addon Library File Write Policy Evidence And Gates

Status: Completed
Last updated: 2026-05-19

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
cargo check -p nako-core -p nako-db -p nako-api -p nako-server -p nako-nfo -p nako-vfs --tests
cargo nextest run -p nako-server addon_side_effect --no-fail-fast
cargo nextest run -p nako-server nfo --no-fail-fast
cargo nextest run -p nako-nfo --no-fail-fast
cargo nextest run -p nako-vfs --no-fail-fast
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
- `crates/nako-nfo/src/export.rs`
- `crates/nako-vfs/src/lib.rs`
- `crates/nako-vfs/src/local.rs`
- `crates/nako-server/src/app/nfo.rs`
- `crates/nako-server/src/app/addons.rs`

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
  - `CONTEXT.md` confirms Library File Write is the Nako-owned boundary for
    subtitles, NFO files, artwork, and sidecar assets, and NFO Export is
    governed by local file-write policy.
  - `crates/nako-core/src/addon.rs` already has `library_file_write` and
    `subtitle_write` permissions, plus media item/source side-effect targets.
  - `crates/nako-server/src/app/addons.rs` currently applies only
    `metadata_write`; unsupported permissions are skipped with safe error
    codes.
  - `crates/nako-nfo/src/export.rs` derives NFO sidecar targets from
    `MediaSource`, preserves existing NFO XML on forced export, and writes via
    `StorageWriteRequest::atomic_replace`.
  - `crates/nako-vfs/src/lib.rs` and `crates/nako-vfs/src/local.rs` expose
    write modes, backup policy, write reports, same-directory atomic replace,
    existing-file backup, and keep-latest pruning.
  - `crates/nako-server/src/app/nfo.rs` already wraps NFO import/export in
    durable jobs, concurrency limiting, writable-backend checks, and redacted
    outbox event payloads.
- Decision: ALFW-030 should implement MediaSource-targeted addon-initiated
  Nako-owned NFO Export as the first Library File Write apply path. The addon
  may request an NFO export intent and write policy, but Nako derives the NFO
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

2026-05-18, ALFW-030 implementation evidence:

- Implemented synchronous MediaSource-targeted `library_file_write` apply for
  Nako-owned NFO Export. The side effect is marked `applied` only after
  first-party NFO/VFS execution completes.
- Added a typed NFO export payload with `file_role: "nfo"` and policy
  `create_missing` or `replace_existing_preserving`. Unknown fields, raw NFO
  payloads, paths, Source Locators, and remote handles are rejected instead of
  consumed.
- Added redacted `apply_report` persistence and API output for Addon Side
  Effects. NFO export reports include aggregate counters only; they do not
  expose `StorageWriteReport`, `StorageUri`, Source Locators, filesystem paths,
  backup URIs, remote handles, or raw payload content.
- Added `NfoService::export_media_source` so Addon apply can reuse the
  first-party NFO sidecar derivation, NFO Round Trip rendering, VFS atomic
  replace, existing-file backup, and retention diagnostics for a single
  Media Source.
- Fixed `LibraryStorageBackend` to forward typed `StorageWriteRequest` writes
  to the underlying backend; previously only string writes were forwarded, so
  NFO export through the wrapped backend fell back to the default unsupported
  method.
- Behavior covered by focused tests:
  - create-missing MediaSource NFO export writes `demo.nfo`, records
    `applied_source: "nfo_export"`, returns a redacted report, and replays
    idempotently without rewriting.
  - replace-existing-preserving updates owned NFO fields, preserves unknown XML,
    creates a backup, and reports only aggregate backup counters.
  - raw NFO payload fields fail safely without writing a sidecar; MediaItem
    targets are rejected before NFO export apply.
- Fresh validation:
  - `CARGO_TARGET_DIR=G:\nako-cargo-target cargo check -p nako-core -p nako-db -p nako-api -p nako-server -p nako-nfo -p nako-vfs --tests`
    passed.
  - `CARGO_TARGET_DIR=G:\nako-cargo-target cargo nextest run -p nako-db addon_side_effect --no-fail-fast`
    passed with 2 tests.
  - `CARGO_TARGET_DIR=G:\nako-cargo-target cargo nextest run -p nako-server addon_side_effect --no-fail-fast`
    passed with 8 selected tests.
  - `CARGO_TARGET_DIR=G:\nako-cargo-target cargo nextest run -p nako-server nfo --no-fail-fast`
    passed with 9 selected tests.
  - `CARGO_TARGET_DIR=G:\nako-cargo-target cargo nextest run -p nako-nfo --no-fail-fast`
    passed with 20 tests.
  - `CARGO_TARGET_DIR=G:\nako-cargo-target cargo nextest run -p nako-vfs --no-fail-fast`
    passed with 28 tests.
  - `cargo fmt --all -- --check` passed.
  - `git diff --check` passed with only Git CRLF normalization warnings for
    the edited files.

2026-05-19, ALFW-040 closeout review and split:

- ALFW-030 was committed as `785ffe6 feat(addons): apply media source nfo
  library file writes`.
- Review result:
  - Workstream compliance has no blocking findings. The shipped behavior
    matches the selected first target: MediaSource-targeted Nako-owned NFO
    Export through `library_file_write`.
  - Code-quality review has no blocking findings. The Addon handler
    authenticates, validates, records, and delegates; it does not own NFO XML
    rendering, storage writes, NFO import persistence, scan-source updates, or
    search-projection ordering.
  - Redaction review has no blocking findings. Responses and stored
    `apply_report` values expose only safe IDs/statuses and aggregate counters,
    not raw payloads, Source Locators, filesystem paths, remote handles, backup
    URIs, `StorageUri`, or `StorageWriteReport`.
- Closeout decision:
  - Close this lane after the first Library File Write path. Do not broaden the
    completed lane with subtitle, arbitrary sidecar, broader NFO, or queued
    execution semantics.
  - Future subtitle file writes need a first-party subtitle/track model,
    language/format validation, conflict policy, and safe report shape.
  - Future arbitrary sidecar asset writes need content-type and target
    derivation rules before accepting addon payloads.
  - Future queued Library File Write execution needs truthful queued/job
    association semantics before `apply_status` can represent deferred work.
- Final closeout gates after ALFW-040 documentation edits:
  - `Get-Content -Raw docs\workstreams\addon-library-file-write-policy\WORKSTREAM.json | ConvertFrom-Json | Out-Null`
    exited 0.
  - `cargo fmt --all -- --check` exited 0.
  - `git diff --check` exited 0. Git reported Windows LF-to-CRLF working-copy
    warnings only; no whitespace errors were reported.
  - `CARGO_TARGET_DIR=G:\nako-cargo-target cargo check -p nako-core -p nako-db -p nako-api -p nako-server -p nako-nfo -p nako-vfs --tests`
    exited 0.
  - `CARGO_TARGET_DIR=G:\nako-cargo-target cargo nextest run -p nako-server addon_side_effect --no-fail-fast`
    exited 0 with 8 selected tests passed.
- ALFW status is now completed. The recommended next addon lane is
  `addon-managed-artwork-artifacts` if poster/backdrop import is the next
  user-visible plugin value; otherwise open a new subtitle-focused Library File
  Write follow-on.
