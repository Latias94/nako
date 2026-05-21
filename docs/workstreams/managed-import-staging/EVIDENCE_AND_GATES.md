# Managed Import Staging — Evidence And Gates

Status: Complete
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
| 2026-05-21 | MIS-020 TDD red gate | `cargo nextest run -p taru-db managed_import --no-fail-fast` | Expected fail. Contract test could not compile because `ManagedImportRepository` was not implemented for SQLite/PostgreSQL/facade yet. |
| 2026-05-21 | MIS-020 implementation gate | `cargo nextest run -p taru-db managed_import --no-fail-fast` | Pass. `sqlite_managed_import_contract_round_trips_artifacts_and_state` proves durable artifact round-trip, source lookup, list filters, staging-manifest reference, and state transition. PostgreSQL paired contract compiles and remains ignored unless `TARU_TEST_POSTGRES_URL` is provided. |
| 2026-05-21 | MIS-020 formatting | `cargo fmt --all -- --check` | Pass. |
| 2026-05-21 | MIS-020 diff hygiene | `git diff --check` | Pass with repository line-ending warnings only; no whitespace errors. |
| 2026-05-21 | MIS-030 server diagnostics | `cargo nextest run -p taru-server managed_import --no-fail-fast` | Pass. Service tests prove create/list diagnostics are redacted, library scoped, enriched from staging manifest facts, reject mutating creation states, and do not create Media Sources. |
| 2026-05-21 | MIS-040 TDD red gate | `cargo nextest run -p taru-server managed_import --no-fail-fast` | Expected fail. New promotion-preview tests could not compile because the core promotion plan model and `preview_promotion_plan` app method did not exist yet. |
| 2026-05-21 | MIS-040 promotion preview gate | `cargo nextest run -p taru-server managed_import --no-fail-fast` | Pass. Six service tests prove non-mutating promotion preview explains destination, hardlink/symlink dry-run status, duplicate hints, NFO authority hint, provider identity hint, explicit blockers, and destination-escape rejection without creating Media Sources or writing/copying/linking/deleting library files. |
| 2026-05-21 | MIS-040 focused verification | `cargo fmt --all -- --check`; `cargo nextest run -p taru-server managed_import --no-fail-fast`; `cargo nextest run -p taru-db managed_import --no-fail-fast`; `cargo nextest run -p taru-vfs link --no-fail-fast`; `python -m json.tool docs/workstreams/managed-import-staging/WORKSTREAM.json`; `git diff --check` | Pass. Formatting, six Managed Import server tests, DB artifact contract, VFS link dry-run tests, workstream JSON validity, and diff hygiene are fresh. `git diff --check` emitted only repository CRLF conversion warnings. |
| 2026-05-21 | MIS-050 split decision | `docs/workstreams/managed-import-staging/DESIGN.md`; `docs/workstreams/link-apply-and-import-promotion/DESIGN.md` | Pass. Actual promotion apply is split to a dedicated follow-on with explicit operator confirmation, plan revalidation, durable audit, rollback, cleanup, VFS-only mutation, catalog consistency, and NFO boundary requirements. |
| 2026-05-21 | MIS-060 closeout | `docs/workstreams/managed-import-staging`; `docs/workstreams/post-rpd-product-hardening`; `docs/workstreams/README.md` | Pass pending final command rerun. Managed Import Staging is closed as a planning/staging lane; the parent umbrella and index route next execution to `link-apply-and-import-promotion`. |

## Evidence Anchors

- `crates/taru-core/src/staging.rs`
- `crates/taru-core/src/managed_import.rs`
- `crates/taru-core/src/repository/managed_import.rs`
- `crates/taru-core/src/repository/vfs.rs`
- `crates/taru-db/src/sqlite/managed_import.rs`
- `crates/taru-db/src/postgres.rs`
- `crates/taru-server/src/app/managed_import.rs`
- `crates/taru-server/src/app/tests/managed_import.rs`
- `crates/taru-server/src/app/storage.rs`
- `crates/taru-server/src/app/staging.rs`
- `docs/workstreams/managed-import-staging/DESIGN.md`
- `docs/workstreams/nfo-link-authority/DESIGN.md`

## Notes

Managed Import Staging must not be treated as playback/probe staging with a new
label. Product import needs artifact identity, target library intent,
promotion-plan diagnostics, acceptance state, rollback, cleanup, and audit.
