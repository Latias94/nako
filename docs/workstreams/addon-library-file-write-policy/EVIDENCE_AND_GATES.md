# Addon Library File Write Policy Evidence And Gates

Status: Proposed
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
cargo nextest run -p taru-vfs --no-fail-fast
cargo fmt --all -- --check
git diff --check
```

Adjust focused filters after ALFW-020 chooses the first concrete target.

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
- `crates/taru-nfo/src/export.rs`
- `crates/taru-vfs/src/lib.rs`
- `crates/taru-vfs/src/local.rs`
- `crates/taru-server/src/app/addons.rs`

## Fresh Evidence

2026-05-18, ALFW-010:

- Workstream opened from APW-060 closeout as the follow-on for subtitle, NFO,
  and sidecar-asset Library File Write behavior.
- This is a planning split only; no file-write runtime behavior changed.
- Fresh validation remains required before marking ALFW-020 or later tasks
  complete.
