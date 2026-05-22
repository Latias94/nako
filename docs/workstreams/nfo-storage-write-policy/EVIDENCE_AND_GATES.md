# NFO Storage Write Policy Evidence And Gates

Status: Completed
Last updated: 2026-05-17

## Baseline Evidence

- M47 preserves unknown NFO XML fields during forced export.
- `NfoService::export_source` currently writes final XML through
  `StorageBackend::write_string`.
- `LocalFsBackend::write_string` currently writes directly to the final file
  path with `fs::write`.
- WebDAV remains read-only for writes in this slice.

## Focused Gates

```powershell
cargo fmt --all -- --check
cargo check -p nako-vfs --tests
cargo nextest run -p nako-vfs --no-fail-fast
cargo check -p nako-nfo --tests
cargo nextest run -p nako-nfo --no-fail-fast
```

## Closeout Gates

```powershell
cargo check --workspace --tests
cargo nextest run --workspace --no-fail-fast
git diff --check
```

## Evidence Log

- 2026-05-17: Workstream opened for M48.
- 2026-05-17: `cargo fmt --all -- --check` passed after formatting the
  touched `nako-vfs` and `nako-nfo` packages.
- 2026-05-17: `cargo check -p nako-vfs --tests` passed.
- 2026-05-17: `cargo nextest run -p nako-vfs --no-fail-fast` passed with 22
  tests.
- 2026-05-17: `cargo check -p nako-nfo --tests` passed.
- 2026-05-17: `cargo nextest run -p nako-nfo --no-fail-fast` passed with 16
  tests.
- 2026-05-17: NFO export now requests `StorageWriteMode::AtomicReplace`, local
  storage implements same-directory temp-file replace, and NFO failures carry
  internal `NfoFailureKind` categories for parse, preservation, unsupported
  atomic write, and storage read/write errors.
- 2026-05-17: `cargo check --workspace --tests` passed.
- 2026-05-17: `cargo nextest run --workspace --no-fail-fast` passed with 305
  tests.
- 2026-05-17: `git diff --check` passed.

## Expected Closeout Evidence

- VFS tests prove local atomic replace writes sidecars safely.
- VFS tests prove unsupported atomic write requests fail explicitly for default
  backends.
- NFO export tests prove local sidecar export uses the explicit write policy.
- NFO export tests prove parse/preservation/write failures carry diagnostic
  categories.
- M47 preservation and import/export round trip tests still pass.
