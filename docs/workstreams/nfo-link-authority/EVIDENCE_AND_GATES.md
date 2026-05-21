# NFO Link Authority — Evidence And Gates

Status: Complete
Last updated: 2026-05-21

## Gate Set

### LNA-020 Link Planning Gate

```powershell
cargo nextest run -p taru-vfs link --no-fail-fast
cargo fmt --all -- --check
git diff --check
```

### Broader Lane Gate

```powershell
cargo nextest run -p taru-vfs --no-fail-fast
cargo nextest run -p taru-nfo --no-fail-fast
cargo nextest run -p taru-db source_duplicate --no-fail-fast
cargo nextest run -p taru-server catalog --no-fail-fast
cargo fmt --all -- --check
git diff --check
```

Use narrower gates while iterating, but record the reason before claiming lane
closeout.

## Evidence Log

| Date | Scope | Command / Evidence | Result |
| --- | --- | --- | --- |
| 2026-05-21 | LNA-010 planning | Reviewed `CONTEXT.md`, ADR 0002/0007/0008, completed NFO write/backup/retention lanes, Source Duplicate Relationship code, and `post-rpd-product-hardening` handoff | Pass. First safe slice is VFS link dry-run diagnostics; actual link mutation remains out of scope. |
| 2026-05-21 | LNA-020 targeted | `cargo nextest run -p taru-vfs link --no-fail-fast` | Pass. 4 link planning tests passed. Local dry-run reports ready, existing target, missing source, and missing parent without creating targets; default backend reports unsupported. |
| 2026-05-21 | LNA-020 red/green | `cargo nextest run -p taru-vfs link --no-fail-fast` after first implementation attempt | Failed once because missing target parent returned a low-level storage IO error instead of a dry-run `TargetParentMissing` plan; fixed by validating target parent non-destructively before canonicalization. |
| 2026-05-21 | LNA-020 package | `cargo nextest run -p taru-vfs --no-fail-fast` | Pass. 32 VFS tests passed. |
| 2026-05-21 | LNA-020 formatting | `cargo fmt --all -- --check` | Pass. |
| 2026-05-21 | LNA-020 diff hygiene | `git diff --check` | Pass. Git reported CRLF conversion warnings only. |
| 2026-05-21 | LNA-030 red/green | `cargo nextest run -p taru-server filesystem_link_duplicate --no-fail-fast` | Failed once because the test did not persist the parent Library row, triggering a foreign-key error before source setup; fixed by inserting the Library fixture. |
| 2026-05-21 | LNA-030 targeted | `cargo nextest run -p taru-server filesystem_link_duplicate --no-fail-fast` | Pass. App service records `FilesystemLink` Source Duplicate Relationship evidence from a VFS link plan and proves Media Source item links remain unchanged. |
| 2026-05-21 | LNA-030 rejection | `cargo nextest run -p taru-server link_plan_status --no-fail-fast` | Pass. Non-evidence link plan statuses are rejected and no Source Duplicate Relationship is inserted. |
| 2026-05-21 | LNA-030 package slice | `cargo nextest run -p taru-server catalog --no-fail-fast` | Pass. 9 catalog-related server tests passed. |
| 2026-05-21 | LNA-030 repository contract | `cargo nextest run -p taru-db source_duplicate --no-fail-fast` | Pass. Source Duplicate Relationship persistence still round-trips without merging items. |
| 2026-05-21 | LNA-030 link regression | `cargo nextest run -p taru-vfs link --no-fail-fast` | Pass. 4 VFS link planning tests still pass after server evidence integration. |
| 2026-05-21 | LNA-030 formatting | `cargo fmt --all -- --check` | Pass after running `cargo fmt --all` to normalize the new catalog app test. |
| 2026-05-21 | LNA-030 diff hygiene | `git diff --check` | Pass. Git reported CRLF conversion warnings only. |
| 2026-05-21 | LNA-040 red/green | `cargo nextest run -p taru-server nfo_authority_preview --no-fail-fast` | Failed once because the new preview module was not re-exported from `taru-nfo`; fixed by exporting preview types. |
| 2026-05-21 | LNA-040 targeted service | `cargo nextest run -p taru-nfo authority_preview --no-fail-fast` | Pass. NFO service preview reports create/skip/update/backup-required decisions and does not write sidecars. |
| 2026-05-21 | LNA-040 targeted app | `cargo nextest run -p taru-server nfo_authority_preview --no-fail-fast` | Pass. NFO app preview explains export create/skip/forced update/policy rejection and import update without mutating files or metadata. |
| 2026-05-21 | LNA-040 package | `cargo nextest run -p taru-nfo --no-fail-fast` | Pass. 24 NFO tests passed. |
| 2026-05-21 | LNA-040 server slice | `cargo nextest run -p taru-server nfo --no-fail-fast` | Pass. 13 NFO-related server tests passed. |
| 2026-05-21 | LNA-040 formatting | `cargo fmt --all -- --check` | Pass. |
| 2026-05-21 | LNA-040 diff hygiene | `git diff --check` | Pass. Git reported CRLF conversion warnings only. |
| 2026-05-21 | LNA-050 split decision | `DESIGN.md`, `MILESTONES.md`, `HANDOFF.md` | Pass. Actual hardlink/symlink apply is split to a follow-on after managed import staging defines promotion, rollback, cleanup, audit, and source duplicate confirmation semantics. |
| 2026-05-21 | LNA-060 closeout | `TODO.md`, `WORKSTREAM.json`, `HANDOFF.md` | Pass. Workstream marked complete; follow-ons are explicit. |
| 2026-05-21 | LNA-060 fresh closeout gate | `cargo nextest run -p taru-vfs link --no-fail-fast`; `cargo nextest run -p taru-db source_duplicate --no-fail-fast`; `cargo nextest run -p taru-server catalog --no-fail-fast`; `cargo nextest run -p taru-nfo --no-fail-fast`; `cargo nextest run -p taru-server nfo --no-fail-fast`; `cargo fmt --all -- --check`; `git diff --check` | Pass. 4 VFS link tests, 1 source duplicate repository test, 9 catalog server tests, 24 NFO crate tests, and 13 NFO server tests passed. `git diff --check` reported CRLF conversion warnings only. |

## Evidence Anchors

- `crates/taru-vfs/src/lib.rs`
- `crates/taru-vfs/src/local.rs`
- `crates/taru-vfs/src/cache.rs`
- `crates/taru-nfo/src/preview.rs`
- `crates/taru-nfo/src/lib.rs`
- `crates/taru-core/src/media/source.rs`
- `crates/taru-core/src/repository/metadata.rs`
- `crates/taru-server/src/app/catalog.rs`
- `crates/taru-server/src/app/nfo.rs`
- `crates/taru-server/src/app/tests/catalog.rs`
- `crates/taru-server/src/app/tests/nfo.rs`
- `docs/workstreams/nfo-link-authority/DESIGN.md`

## Notes

Do not claim link authority complete merely because `StorageCapabilities` has
`LINKABLE`. The lane requires typed dry-run evidence and later duplicate/NFO
authority diagnostics.
