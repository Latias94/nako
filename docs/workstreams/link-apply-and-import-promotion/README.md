# Link Apply And Import Promotion

Status: Active

This workstream is the follow-on split from `managed-import-staging` and
`nfo-link-authority`. It owns the first safe mutation path from a Managed Import
promotion preview into a Media Library: operator-confirmed copy, hardlink, or
symlink apply with durable audit, rollback/cleanup semantics, catalog
consistency, and redacted diagnostics.

The lane deliberately starts with acceptance/audit and storage mutation seams,
not torrent/Usenet acquisition, background downloader orchestration, or Admin UI
polish. A promotion preview is explanatory only; this lane defines what makes a
specific apply command authorized, idempotent, reversible, and observable.

## Goals

- Add durable promotion acceptance/apply audit records separate from preview
  DTOs.
- Revalidate promotion plan facts before any storage mutation.
- Apply copy/hardlink/symlink through VFS/storage backends only.
- Commit Media Source and Source Duplicate Relationship state only after the
  target locator is durable.
- Record rollback or cleanup-pending evidence after partial failure.
- Keep NFO import/export side effects explicit and out of the first apply slice
  unless their backup/authority gates are part of the accepted operation.

## Non-Goals

- No generic downloader, torrent, Usenet, browser scraping, or watch-folder
  runtime in the first slice.
- No automatic apply from a preview without operator confirmation.
- No move/delete source operation until rollback and source-retention semantics
  are proven.
- No direct OS path manipulation in server code.
- No Public Client API exposure.
- No AI or Addon autonomous library writes.

## Authoritative Docs

- [Design](DESIGN.md)
- [TODO](TODO.md)
- [Milestones](MILESTONES.md)
- [Evidence and gates](EVIDENCE_AND_GATES.md)
- [Handoff](HANDOFF.md)

## Related Workstreams

- [managed-import-staging](../managed-import-staging/README.md)
- [nfo-link-authority](../nfo-link-authority/README.md)
- [post-rpd-product-hardening](../post-rpd-product-hardening/README.md)
- [addon-library-file-write-policy](../addon-library-file-write-policy/README.md)
- [managed-artwork-fetch-artifact-storage](../managed-artwork-fetch-artifact-storage/README.md)