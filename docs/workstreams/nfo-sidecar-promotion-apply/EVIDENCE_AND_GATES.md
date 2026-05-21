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

## Evidence Anchors

- `docs/workstreams/nfo-sidecar-promotion-apply/DESIGN.md`
- `docs/workstreams/nfo-sidecar-promotion-apply/TODO.md`
- `docs/workstreams/link-apply-and-import-promotion/DESIGN.md`
- `crates/taru-core/src/nfo_sidecar_apply.rs`
- `crates/taru-core/src/repository/nfo_sidecar_apply.rs`
- `crates/taru-db/src/sqlite/nfo_sidecar_apply.rs`
- `crates/taru-db/migrations/0033_nfo_sidecar_applies.sql`
- `crates/taru-db/migrations/postgres/0005_nfo_sidecar_applies.sql`
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
