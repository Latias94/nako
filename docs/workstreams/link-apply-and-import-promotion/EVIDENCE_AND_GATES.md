# Link Apply And Import Promotion — Evidence And Gates

Status: Active
Last updated: 2026-05-21

## Gate Set

### LAIP-010 Planning Gate

```powershell
python -m json.tool docs/workstreams/link-apply-and-import-promotion/WORKSTREAM.json
python -m json.tool docs/workstreams/post-rpd-product-hardening/WORKSTREAM.json
git diff --check
```

### First Implementation Gate

```powershell
cargo nextest run -p taru-db promotion_apply --no-fail-fast
cargo fmt --all -- --check
git diff --check
```

### Broader Lane Gate

```powershell
cargo nextest run -p taru-db promotion_apply --no-fail-fast
cargo nextest run -p taru-server managed_import --no-fail-fast
cargo nextest run -p taru-vfs link --no-fail-fast
cargo fmt --all -- --check
git diff --check
```

Use narrower gates during iteration, but record skipped broader gates before
claiming lane closeout.

## Evidence Log

| Date | Scope | Command / Evidence | Result |
| --- | --- | --- | --- |
| 2026-05-21 | LAIP-010 planning | Reviewed Managed Import Staging MIS-050/MIS-060, NFO Link Authority closeout, VFS link planning, Managed Import promotion preview, and addon file-write policy boundaries | Pass. First safe slice is durable promotion acceptance/audit, not storage mutation or downloader implementation. |
| 2026-05-21 | LAIP-020 TDD red gate | `cargo nextest run -p taru-db promotion_apply --no-fail-fast` | Expected fail. New contract test could not compile because `ManagedImportPromotionApplyId`, `ManagedImportPromotionApplyState`, `NewManagedImportPromotionApply`, and repository methods did not exist yet. |
| 2026-05-21 | LAIP-020 implementation gate | `cargo nextest run -p taru-db promotion_apply --no-fail-fast` | Pass. `sqlite_promotion_apply_contract_round_trips_acceptance_and_audit` proves durable apply/audit round-trip, idempotency-key lookup, artifact-scoped listing, accepted state update, cleanup-pending audit update, and missing-row state update behavior. PostgreSQL paired contract compiles and remains ignored unless `TARU_TEST_POSTGRES_URL` is provided. |
| 2026-05-21 | LAIP-020 focused verification | `cargo fmt --all -- --check`; `cargo nextest run -p taru-db managed_import --no-fail-fast`; `cargo nextest run -p taru-db promotion_apply --no-fail-fast`; `python -m json.tool docs/workstreams/managed-import-staging/WORKSTREAM.json`; `python -m json.tool docs/workstreams/link-apply-and-import-promotion/WORKSTREAM.json`; `python -m json.tool docs/workstreams/post-rpd-product-hardening/WORKSTREAM.json`; `git diff --check` | Pass. Formatting, Managed Import artifact contract, promotion apply contract, workstream JSON validity, and diff hygiene are fresh. `git diff --check` emitted only repository CRLF conversion warnings. |
| 2026-05-21 | LAIP-030 TDD red gate | `cargo nextest run -p taru-server managed_import --no-fail-fast` | Expected fail. New app-service acceptance tests could not compile because `AcceptManagedImportPromotionRequest` and `ManagedImportAppService::accept_promotion` did not exist yet. |
| 2026-05-21 | LAIP-030 implementation gate | `cargo nextest run -p taru-server managed_import --no-fail-fast` | Pass. Eight Managed Import app tests prove explicit promotion acceptance, idempotent replay, mismatched idempotency-key rejection, blocked-plan rejection, redacted diagnostics, and no library-file or Media Source mutation before storage apply tasks. |
| 2026-05-21 | LAIP-040 TDD red gate | `cargo nextest run -p taru-vfs apply --no-fail-fast` | Expected fail. New VFS apply tests could not compile because `StorageApplyKind`, `StorageApplyRequest`, `StorageApplyStatus`, `StorageBackend::apply`, and local apply behavior did not exist yet. |
| 2026-05-21 | LAIP-040 implementation gate | `cargo nextest run -p taru-vfs apply --no-fail-fast`; `cargo nextest run -p taru-vfs local_backend_applies --no-fail-fast`; `cargo nextest run -p taru-vfs link --no-fail-fast`; `cargo nextest run -p taru-vfs --no-fail-fast` | Pass. VFS tests prove default unsupported apply behavior, local copy apply without raw OS path reports, local hardlink apply through a ready plan, symlink apply or typed platform failure, target-exists no-overwrite behavior, security-violation no-mutation behavior, and full `taru-vfs` regression coverage. |
| 2026-05-21 | LAIP-040 verification gate | `cargo fmt --all -- --check`; `cargo nextest run -p taru-vfs link --no-fail-fast`; `cargo nextest run -p taru-vfs apply --no-fail-fast`; `cargo nextest run -p taru-vfs local_backend_applies --no-fail-fast`; `cargo nextest run -p taru-vfs --no-fail-fast`; `python -m json.tool docs/workstreams/link-apply-and-import-promotion/WORKSTREAM.json`; `git diff --check` | Pass. Fresh verification proves formatting, link/apply focused behavior, full `taru-vfs` regression coverage, workstream JSON validity, and diff hygiene. |
| 2026-05-21 | LAIP-050 TDD red gate | `cargo nextest run -p taru-server managed_import_applies_accepted_promotion_after_storage_target_is_durable --no-fail-fast` | Expected fail. New apply test could not compile because `ApplyManagedImportPromotionRequest` and `ManagedImportAppService::apply_promotion` did not exist yet. |
| 2026-05-21 | LAIP-050 implementation gate | `cargo nextest run -p taru-server managed_import_apply --no-fail-fast`; `cargo nextest run -p taru-server managed_import --no-fail-fast` | Pass. Server tests prove successful accepted promotion apply, VFS-mediated storage apply boundary, stale acceptance and already-cataloged destination rejection before storage mutation, source-missing failure audit without Media Source writes, promoted replay, duplicate relationship suggestions, redacted diagnostics, artifact promotion state, and catalog writes only after target creation. |
| 2026-05-21 | LAIP-050 verification gate | `cargo fmt --all -- --check`; `cargo nextest run -p taru-server managed_import --no-fail-fast`; `cargo nextest run -p taru-db promotion_apply --no-fail-fast`; `cargo nextest run -p taru-vfs link --no-fail-fast`; `cargo nextest run -p taru-vfs apply --no-fail-fast`; `python -m json.tool docs/workstreams/link-apply-and-import-promotion/WORKSTREAM.json`; `git diff --check` | Pass. Fresh verification proves formatting, Managed Import apply/acceptance regression coverage, promotion apply persistence contract coverage, VFS link/apply boundary regression coverage, workstream JSON validity, and diff hygiene. `git diff --check` emitted only repository CRLF conversion warnings. |

## Evidence Anchors

- `docs/workstreams/link-apply-and-import-promotion/DESIGN.md`
- `docs/workstreams/link-apply-and-import-promotion/TODO.md`
- `docs/workstreams/managed-import-staging/DESIGN.md`
- `docs/workstreams/nfo-link-authority/DESIGN.md`
- `crates/taru-core/src/managed_import.rs`
- `crates/taru-core/src/repository/managed_import.rs`
- `crates/taru-db/src/contract_tests.rs`
- `crates/taru-db/migrations/0032_managed_import_promotion_applies.sql`
- `crates/taru-db/migrations/postgres/0004_managed_import_promotion_applies.sql`
- `crates/taru-db/src/sqlite/managed_import.rs`
- `crates/taru-db/src/postgres.rs`
- `crates/taru-vfs/src/lib.rs`
- `crates/taru-vfs/src/local.rs`
- `crates/taru-server/src/app/managed_import.rs`
- `crates/taru-server/src/app/tests/managed_import.rs`

## Notes

A promotion preview is not authorization to mutate storage. Apply must be an
explicit accepted command with durable audit, idempotency, revalidation,
rollback/cleanup, and catalog consistency gates.
