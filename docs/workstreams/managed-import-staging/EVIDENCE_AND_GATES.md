# Managed Import Staging — Evidence And Gates

Status: Active
Last updated: 2026-05-21

## Gate Set

### MIS-010 Planning Gate

```powershell
python -m json.tool docs/workstreams/managed-import-staging/WORKSTREAM.json
python -m json.tool docs/workstreams/post-rpd-product-hardening/WORKSTREAM.json
git diff --check
```

### First Implementation Gate

```powershell
cargo nextest run -p taru-db managed_import --no-fail-fast
cargo fmt --all -- --check
git diff --check
```

### Broader Lane Gate

```powershell
cargo nextest run -p taru-db managed_import --no-fail-fast
cargo nextest run -p taru-server managed_import --no-fail-fast
cargo nextest run -p taru-vfs link --no-fail-fast
cargo nextest run -p taru-nfo --no-fail-fast
cargo fmt --all -- --check
git diff --check
```

Use narrower gates during iteration, but record skipped broader gates before
claiming lane closeout.

## Evidence Log

| Date | Scope | Command / Evidence | Result |
| --- | --- | --- | --- |
| 2026-05-21 | MIS-010 planning | Reviewed `CONTEXT.md`, post-RPD closeout, NFO/link authority closeout, existing VFS staging manifests, staging cleanup/service code, and DB staging contracts | Pass. First safe slice is durable Managed Import artifact domain/schema, not downloader implementation or promotion apply. |

## Evidence Anchors

- `crates/taru-core/src/staging.rs`
- `crates/taru-core/src/repository/vfs.rs`
- `crates/taru-server/src/app/staging.rs`
- `docs/workstreams/managed-import-staging/DESIGN.md`
- `docs/workstreams/nfo-link-authority/DESIGN.md`

## Notes

Managed Import Staging must not be treated as playback/probe staging with a new
label. Product import needs artifact identity, target library intent,
promotion-plan diagnostics, acceptance state, rollback, cleanup, and audit.