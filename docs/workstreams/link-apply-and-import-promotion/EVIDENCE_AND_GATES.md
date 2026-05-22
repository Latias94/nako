# Link Apply And Import Promotion — Evidence And Gates

Status: Complete
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
cargo nextest run -p nako-db promotion_apply --no-fail-fast
cargo fmt --all -- --check
git diff --check
```

### Broader Lane Gate

```powershell
cargo nextest run -p nako-db promotion_apply --no-fail-fast
cargo nextest run -p nako-server managed_import --no-fail-fast
cargo nextest run -p nako-vfs link --no-fail-fast
cargo fmt --all -- --check
git diff --check
```

Use narrower gates during iteration, but record skipped broader gates before
claiming lane closeout.

## Evidence Log

| Date | Scope | Command / Evidence | Result |
| --- | --- | --- | --- |
| 2026-05-21 | LAIP-010 planning | Reviewed Managed Import Staging MIS-050/MIS-060, NFO Link Authority closeout, VFS link planning, Managed Import promotion preview, and addon file-write policy boundaries | Pass. First safe slice is durable promotion acceptance/audit, not storage mutation or downloader implementation. |
| 2026-05-21 | LAIP-020 TDD red gate | `cargo nextest run -p nako-db promotion_apply --no-fail-fast` | Expected fail. New contract test could not compile because `ManagedImportPromotionApplyId`, `ManagedImportPromotionApplyState`, `NewManagedImportPromotionApply`, and repository methods did not exist yet. |
| 2026-05-21 | LAIP-020 implementation gate | `cargo nextest run -p nako-db promotion_apply --no-fail-fast` | Pass. `sqlite_promotion_apply_contract_round_trips_acceptance_and_audit` proves durable apply/audit round-trip, idempotency-key lookup, artifact-scoped listing, accepted state update, cleanup-pending audit update, and missing-row state update behavior. PostgreSQL paired contract compiles and remains ignored unless `NAKO_TEST_POSTGRES_URL` is provided. |
| 2026-05-21 | LAIP-020 focused verification | `cargo fmt --all -- --check`; `cargo nextest run -p nako-db managed_import --no-fail-fast`; `cargo nextest run -p nako-db promotion_apply --no-fail-fast`; `python -m json.tool docs/workstreams/managed-import-staging/WORKSTREAM.json`; `python -m json.tool docs/workstreams/link-apply-and-import-promotion/WORKSTREAM.json`; `python -m json.tool docs/workstreams/post-rpd-product-hardening/WORKSTREAM.json`; `git diff --check` | Pass. Formatting, Managed Import artifact contract, promotion apply contract, workstream JSON validity, and diff hygiene are fresh. `git diff --check` emitted only repository CRLF conversion warnings. |
| 2026-05-21 | LAIP-030 TDD red gate | `cargo nextest run -p nako-server managed_import --no-fail-fast` | Expected fail. New app-service acceptance tests could not compile because `AcceptManagedImportPromotionRequest` and `ManagedImportAppService::accept_promotion` did not exist yet. |
| 2026-05-21 | LAIP-030 implementation gate | `cargo nextest run -p nako-server managed_import --no-fail-fast` | Pass. Eight Managed Import app tests prove explicit promotion acceptance, idempotent replay, mismatched idempotency-key rejection, blocked-plan rejection, redacted diagnostics, and no library-file or Media Source mutation before storage apply tasks. |
| 2026-05-21 | LAIP-040 TDD red gate | `cargo nextest run -p nako-vfs apply --no-fail-fast` | Expected fail. New VFS apply tests could not compile because `StorageApplyKind`, `StorageApplyRequest`, `StorageApplyStatus`, `StorageBackend::apply`, and local apply behavior did not exist yet. |
| 2026-05-21 | LAIP-040 implementation gate | `cargo nextest run -p nako-vfs apply --no-fail-fast`; `cargo nextest run -p nako-vfs local_backend_applies --no-fail-fast`; `cargo nextest run -p nako-vfs link --no-fail-fast`; `cargo nextest run -p nako-vfs --no-fail-fast` | Pass. VFS tests prove default unsupported apply behavior, local copy apply without raw OS path reports, local hardlink apply through a ready plan, symlink apply or typed platform failure, target-exists no-overwrite behavior, security-violation no-mutation behavior, and full `nako-vfs` regression coverage. |
| 2026-05-21 | LAIP-040 verification gate | `cargo fmt --all -- --check`; `cargo nextest run -p nako-vfs link --no-fail-fast`; `cargo nextest run -p nako-vfs apply --no-fail-fast`; `cargo nextest run -p nako-vfs local_backend_applies --no-fail-fast`; `cargo nextest run -p nako-vfs --no-fail-fast`; `python -m json.tool docs/workstreams/link-apply-and-import-promotion/WORKSTREAM.json`; `git diff --check` | Pass. Fresh verification proves formatting, link/apply focused behavior, full `nako-vfs` regression coverage, workstream JSON validity, and diff hygiene. |
| 2026-05-21 | LAIP-050 TDD red gate | `cargo nextest run -p nako-server managed_import_applies_accepted_promotion_after_storage_target_is_durable --no-fail-fast` | Expected fail. New apply test could not compile because `ApplyManagedImportPromotionRequest` and `ManagedImportAppService::apply_promotion` did not exist yet. |
| 2026-05-21 | LAIP-050 implementation gate | `cargo nextest run -p nako-server managed_import_apply --no-fail-fast`; `cargo nextest run -p nako-server managed_import --no-fail-fast` | Pass. Server tests prove successful accepted promotion apply, VFS-mediated storage apply boundary, stale acceptance and already-cataloged destination rejection before storage mutation, source-missing failure audit without Media Source writes, promoted replay, duplicate relationship suggestions, redacted diagnostics, artifact promotion state, and catalog writes only after target creation. |
| 2026-05-21 | LAIP-050 verification gate | `cargo fmt --all -- --check`; `cargo nextest run -p nako-server managed_import --no-fail-fast`; `cargo nextest run -p nako-db promotion_apply --no-fail-fast`; `cargo nextest run -p nako-vfs link --no-fail-fast`; `cargo nextest run -p nako-vfs apply --no-fail-fast`; `python -m json.tool docs/workstreams/link-apply-and-import-promotion/WORKSTREAM.json`; `git diff --check` | Pass. Fresh verification proves formatting, Managed Import apply/acceptance regression coverage, promotion apply persistence contract coverage, VFS link/apply boundary regression coverage, workstream JSON validity, and diff hygiene. `git diff --check` emitted only repository CRLF conversion warnings. |
| 2026-05-21 | LAIP-060 TDD red gate | `cargo nextest run -p nako-vfs cleanup --no-fail-fast`; `cargo nextest run -p nako-server managed_import_apply_cleans_storage_target_when_catalog_commit_fails managed_import_apply_records_cleanup_pending_when_storage_cleanup_is_unsupported --no-fail-fast` | Expected fail. New VFS cleanup tests could not compile until `StorageCleanupRequest`, `StorageCleanupStatus`, `StorageCleanupReport`, and `StorageBackend::cleanup` existed. New server tests could not compile until post-storage catalog failure injection and cleanup audit orchestration existed. |
| 2026-05-21 | LAIP-060 implementation gate | `cargo nextest run -p nako-vfs cleanup --no-fail-fast`; `cargo nextest run -p nako-server managed_import_apply_cleans_storage_target_when_catalog_commit_fails managed_import_apply_records_cleanup_pending_when_storage_cleanup_is_unsupported --no-fail-fast`; `cargo nextest run -p nako-server managed_import --no-fail-fast` | Pass. VFS tests prove local file cleanup without OS path exposure, missing target reporting, directory refusal without mutation, security-violation no-mutation behavior, and default unsupported cleanup. Server tests prove injected catalog failure after target creation records cleanup-complete when local cleanup succeeds, records cleanup-pending when cleanup is unsupported, replays cleanup terminal states, keeps Media Source writes empty for injected pre-catalog failures, and never marks failed artifacts promoted. |
| 2026-05-21 | LAIP-060 verification gate | `cargo fmt --all -- --check`; `cargo nextest run -p nako-vfs cleanup --no-fail-fast`; `cargo nextest run -p nako-vfs --no-fail-fast`; `cargo nextest run -p nako-server managed_import --no-fail-fast`; `cargo nextest run -p nako-db promotion_apply --no-fail-fast`; `python -m json.tool docs/workstreams/link-apply-and-import-promotion/WORKSTREAM.json`; `git diff --check` | Pass. Fresh verification proves formatting, focused cleanup behavior, full `nako-vfs` regression coverage, Managed Import apply/cleanup regression coverage, promotion apply persistence contract coverage, workstream JSON validity, and diff hygiene. `git diff --check` emitted only repository CRLF conversion warnings. |
| 2026-05-21 | LAIP-070 split decision | `docs/workstreams/link-apply-and-import-promotion/DESIGN.md`; `docs/workstreams/nfo-sidecar-promotion-apply/DESIGN.md` | Pass. NFO sidecar import/export mutation is split to a dedicated accepted Library File Write and metadata-authority lane. LAIP remains focused on staged artifact promotion, VFS-mediated target creation, catalog commit ordering, duplicate evidence, and cleanup audit. |
| 2026-05-21 | LAIP-070 planning verification | `python -m json.tool docs/workstreams/link-apply-and-import-promotion/WORKSTREAM.json`; `python -m json.tool docs/workstreams/nfo-sidecar-promotion-apply/WORKSTREAM.json`; `python -m json.tool docs/workstreams/post-rpd-product-hardening/WORKSTREAM.json`; `git diff --check` | Pass. Workstream JSON files are valid and diff hygiene passed. `git diff --check` emitted only repository CRLF conversion warnings. |
| 2026-05-21 | LAIP-080 closeout verification | `cargo fmt --all -- --check`; `cargo nextest run -p nako-db promotion_apply --no-fail-fast`; `cargo nextest run -p nako-vfs cleanup --no-fail-fast`; `cargo nextest run -p nako-server managed_import --no-fail-fast`; `cargo nextest run -p nako-vfs link --no-fail-fast`; `python -m json.tool docs/workstreams/link-apply-and-import-promotion/WORKSTREAM.json`; `python -m json.tool docs/workstreams/post-rpd-product-hardening/WORKSTREAM.json`; `python -m json.tool docs/workstreams/nfo-sidecar-promotion-apply/WORKSTREAM.json`; `git diff --check` | Pass. Fresh closeout evidence proves formatting, promotion apply persistence, VFS cleanup, Managed Import apply/cleanup orchestration, VFS link planning/apply regression coverage, workstream JSON validity, and diff hygiene. |

## Evidence Anchors

- `docs/workstreams/link-apply-and-import-promotion/DESIGN.md`
- `docs/workstreams/link-apply-and-import-promotion/TODO.md`
- `docs/workstreams/managed-import-staging/DESIGN.md`
- `docs/workstreams/nfo-link-authority/DESIGN.md`
- `docs/workstreams/nfo-sidecar-promotion-apply/DESIGN.md`
- `crates/nako-core/src/managed_import.rs`
- `crates/nako-core/src/repository/managed_import.rs`
- `crates/nako-db/src/contract_tests.rs`
- `crates/nako-db/migrations/0032_managed_import_promotion_applies.sql`
- `crates/nako-db/migrations/postgres/0004_managed_import_promotion_applies.sql`
- `crates/nako-db/src/sqlite/managed_import.rs`
- `crates/nako-db/src/postgres.rs`
- `crates/nako-vfs/src/lib.rs`
- `crates/nako-vfs/src/local.rs`
- `crates/nako-vfs/src/cache.rs`
- `crates/nako-server/src/app/storage.rs`
- `crates/nako-server/src/app/managed_import.rs`
- `crates/nako-server/src/app/tests/managed_import.rs`

## Notes

A promotion preview is not authorization to mutate storage. Apply must be an
explicit accepted command with durable audit, idempotency, revalidation,
rollback/cleanup, and catalog consistency gates.
