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

## Evidence Anchors

- `docs/workstreams/nfo-sidecar-promotion-apply/DESIGN.md`
- `docs/workstreams/nfo-sidecar-promotion-apply/TODO.md`
- `docs/workstreams/link-apply-and-import-promotion/DESIGN.md`
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
