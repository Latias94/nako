# NFO Sidecar Promotion Apply — Evidence And Gates

Status: Active
Last updated: 2026-05-21

## Gate Set

### Planning Gate

```powershell
python -m json.tool docs/workstreams/nfo-sidecar-promotion-apply/WORKSTREAM.json
python -m json.tool docs/workstreams/link-apply-and-import-promotion/WORKSTREAM.json
python -m json.tool docs/workstreams/post-rpd-product-hardening/WORKSTREAM.json
git diff --check
```

### First Implementation Gate

```powershell
cargo nextest run -p taru-db nfo_sidecar_apply --no-fail-fast
cargo fmt --all -- --check
git diff --check
```

### Broader Lane Gate

```powershell
cargo nextest run -p taru-nfo --no-fail-fast
cargo nextest run -p taru-vfs nfo --no-fail-fast
cargo nextest run -p taru-server nfo --no-fail-fast
cargo nextest run -p taru-db nfo_sidecar_apply --no-fail-fast
cargo fmt --all -- --check
git diff --check
```

Use narrower gates during iteration, but record skipped broader gates before
claiming lane closeout.

## Evidence Log

| Date | Scope | Command / Evidence | Result |
| --- | --- | --- | --- |
| 2026-05-21 | NSPA-010 planning | `docs/workstreams/link-apply-and-import-promotion/DESIGN.md`; `docs/workstreams/nfo-sidecar-promotion-apply/DESIGN.md`; `docs/workstreams/nfo-sidecar-promotion-apply/TODO.md` | Pass. NFO sidecar import/export mutation is split from Managed Import promotion and given its own accepted Library File Write / metadata-authority lane. |
| 2026-05-21 | NSPA-010 planning verification | `python -m json.tool docs/workstreams/nfo-sidecar-promotion-apply/WORKSTREAM.json`; `python -m json.tool docs/workstreams/link-apply-and-import-promotion/WORKSTREAM.json`; `python -m json.tool docs/workstreams/post-rpd-product-hardening/WORKSTREAM.json`; `git diff --check` | Pass. Workstream JSON files are valid and diff hygiene passed. `git diff --check` emitted only repository CRLF conversion warnings. |
| 2026-05-21 | NSPA-020 red gate | `cargo nextest run -p taru-db nfo_sidecar_apply --no-fail-fast` | Expected fail. New backend-neutral contract could not compile before NFO sidecar apply domain/repository types existed. |
| 2026-05-21 | NSPA-020 implementation | `crates/taru-core/src/nfo_sidecar_apply.rs`; `crates/taru-core/src/repository/nfo_sidecar_apply.rs`; `crates/taru-db/migrations/0033_nfo_sidecar_applies.sql`; `crates/taru-db/migrations/postgres/0005_nfo_sidecar_applies.sql`; SQLite/PostgreSQL adapters and facade dispatch | Pass. Durable sidecar apply records now persist accepted preview snapshots, idempotency keys, operation/state, policy version, and redacted audit outcome fields. |
| 2026-05-21 | NSPA-020 focused verification | `cargo nextest run -p taru-db nfo_sidecar_apply --no-fail-fast`; `cargo nextest run -p taru-db promotion_apply --no-fail-fast`; `cargo fmt --all -- --check`; `git diff --check`; `python -m json.tool docs/workstreams/nfo-sidecar-promotion-apply/WORKSTREAM.json` | Pass. Focused NFO sidecar apply and Managed Import promotion apply regression tests passed. Formatting, JSON validity, and diff hygiene passed; `git diff --check` emitted only repository CRLF conversion warnings. |
| 2026-05-21 | NSPA-030 red gate | `cargo nextest run -p taru-server nfo_sidecar_apply --no-fail-fast` | Expected fail. New server behavior tests could not compile before `AcceptNfoSidecarApplyRequest` and `accept_sidecar_apply` existed. |
| 2026-05-21 | NSPA-030 implementation | `crates/taru-server/src/app/nfo.rs`; `crates/taru-server/src/app/tests/nfo.rs` | Pass. Server now accepts a current NFO authority preview, stores a durable accepted sidecar apply audit, replays matching idempotency keys, rejects mismatched/stale accepts, and exposes redacted diagnostics without writing sidecars or mutating canonical metadata. |
| 2026-05-21 | NSPA-030 focused verification | `cargo nextest run -p taru-server nfo_sidecar_apply --no-fail-fast`; `cargo nextest run -p taru-server nfo_authority_preview --no-fail-fast`; `cargo nextest run -p taru-server nfo --no-fail-fast`; `cargo nextest run -p taru-db nfo_sidecar_apply --no-fail-fast`; `cargo fmt --all -- --check`; `python -m json.tool docs/workstreams/nfo-sidecar-promotion-apply/WORKSTREAM.json`; `git diff --check` | Pass. Server NFO acceptance, existing NFO preview/import/export routes, durable sidecar apply persistence, formatting, JSON validity, and diff hygiene passed. `git diff --check` emitted only repository CRLF conversion warnings. |
| 2026-05-21 | NSPA-030 broader prerequisite check | `cargo nextest run -p taru-nfo --no-fail-fast`; `cargo nextest run -p taru-vfs --no-fail-fast`; `cargo nextest run -p taru-vfs nfo --no-fail-fast` | Mixed. Full `taru-nfo` and full `taru-vfs` passed. Filtered `taru-vfs nfo` returned no matching tests and exited with nextest's no-tests error; this is a filter mismatch, not a failing test. |
| 2026-05-21 | NSPA-040 red gate | `cargo nextest run -p taru-server nfo_sidecar_apply_exports_accepted_create_preview_and_commits_audit --no-fail-fast` | Expected fail. New export apply test could not compile before `ApplyNfoSidecarApplyRequest` and `apply_sidecar_apply` existed. |
| 2026-05-21 | NSPA-040 implementation | `crates/taru-server/src/app/nfo.rs`; `crates/taru-server/src/app/tests/nfo.rs`; existing `taru-nfo` export source orchestration; existing VFS atomic backup write APIs | Pass. Accepted export sidecar apply now revalidates the current preview, writes through `taru-nfo` and VFS, commits audit state, idempotently replays committed applies, rejects stale apply before overwrite, and records redacted backup/retention diagnostics. |
| 2026-05-21 | NSPA-040 focused verification | `cargo nextest run -p taru-server nfo_sidecar_apply --no-fail-fast`; `cargo nextest run -p taru-server nfo --no-fail-fast`; `cargo nextest run -p taru-nfo --no-fail-fast`; `cargo nextest run -p taru-vfs --no-fail-fast`; `cargo nextest run -p taru-db nfo_sidecar_apply --no-fail-fast`; `cargo fmt --all -- --check`; `python -m json.tool docs/workstreams/nfo-sidecar-promotion-apply/WORKSTREAM.json`; `git diff --check` | Pass. Export create apply, forced update with backup/retention diagnostics, stale apply rejection, existing NFO flows, NFO/VFS lower-level write behavior, durable persistence, formatting, JSON validity, and diff hygiene passed. `git diff --check` emitted only repository CRLF conversion warnings. |
| 2026-05-21 | NSPA-050 red gate | `cargo nextest run -p taru-server nfo_sidecar_apply_imports_accepted_sidecar_into_metadata_and_locks --no-fail-fast` | Expected fail. Accepted import apply was still rejected with `Unsupported("NFO sidecar apply currently supports export sidecar records only")`. |
| 2026-05-21 | NSPA-050 implementation | `crates/taru-nfo/src/import.rs`; `crates/taru-nfo/src/preview.rs`; `crates/taru-nfo/src/summary.rs`; `crates/taru-server/src/app/nfo.rs`; `crates/taru-server/src/app/tests/nfo.rs` | Pass. Accepted import sidecar apply now revalidates content-fingerprinted preview facts, reads through `taru-nfo`, applies canonical metadata/local authority through repository boundaries, respects user locks, confirms provisional hierarchy, keeps sidecars unchanged, and records redacted audit outcomes. |
| 2026-05-21 | NSPA-050 focused verification | `cargo nextest run -p taru-server nfo_sidecar_apply --no-fail-fast`; `cargo nextest run -p taru-server nfo --no-fail-fast`; `cargo nextest run -p taru-nfo --no-fail-fast`; `cargo nextest run -p taru-db nfo_sidecar_apply --no-fail-fast`; `cargo fmt --all -- --check`; `python -m json.tool docs/workstreams/nfo-sidecar-promotion-apply/WORKSTREAM.json`; `git diff --check` | Pass. Import/export sidecar apply acceptance, import commit, stale import content rejection, user-lock preservation, hierarchy confirmation, existing NFO flows, lower-level NFO behavior, durable persistence, formatting, JSON validity, and diff hygiene passed. `git diff --check` emitted only repository CRLF conversion warnings. |

## Evidence Anchors

- `docs/workstreams/nfo-sidecar-promotion-apply/DESIGN.md`
- `docs/workstreams/nfo-sidecar-promotion-apply/TODO.md`
- `docs/workstreams/link-apply-and-import-promotion/DESIGN.md`
- `crates/taru-core/src/nfo_sidecar_apply.rs`
- `crates/taru-core/src/repository/nfo_sidecar_apply.rs`
- `crates/taru-db/src/sqlite/nfo_sidecar_apply.rs`
- `crates/taru-db/migrations/0033_nfo_sidecar_applies.sql`
- `crates/taru-db/migrations/postgres/0005_nfo_sidecar_applies.sql`
- `crates/taru-nfo/src/import.rs`
- `crates/taru-nfo/src/preview.rs`
- `crates/taru-nfo/src/summary.rs`
- `crates/taru-server/src/app/nfo.rs`
- `crates/taru-server/src/app/tests/nfo.rs`
- `docs/workstreams/nfo-link-authority/DESIGN.md`
- `docs/workstreams/nfo-round-trip-preservation/DESIGN.md`
- `docs/workstreams/nfo-storage-write-policy/DESIGN.md`
- `docs/workstreams/nfo-sidecar-backup-policy/DESIGN.md`
- `docs/workstreams/nfo-backup-retention-diagnostics/DESIGN.md`
- `CONTEXT.md`
- `docs/adr/0007-metadata-merge-policy-and-local-authority.md`
- `docs/adr/0008-nfo-as-local-metadata-boundary.md`

## Notes

NFO preview is not authorization. Apply must be an explicit accepted command
with durable audit, idempotency, preview revalidation, local-authority rules,
backup/rollback or repair-pending outcomes, and redacted diagnostics.
