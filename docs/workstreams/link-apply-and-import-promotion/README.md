# Link Apply And Import Promotion

Status: Complete

Closed on 2026-05-21. LAIP-070 split NFO sidecar import/export mutation to
`nfo-sidecar-promotion-apply`; LAIP-080 closed this lane after fresh promotion
apply, cleanup, formatting, and diff gates.

This workstream is the follow-on split from `managed-import-staging` and
`nfo-link-authority`. It owns the first safe mutation path from a Managed Import
promotion preview into a Media Library: operator-confirmed copy, hardlink, or
symlink apply with durable audit, rollback/cleanup semantics, catalog
consistency, and redacted diagnostics.

The lane deliberately starts with acceptance/audit and storage mutation seams,
not torrent/Usenet acquisition, background downloader orchestration, or Admin UI
polish. A promotion preview is explanatory only; this lane defines what makes a
specific apply command authorized, idempotent, reversible, and observable.

## Shipped Outcome

- Durable promotion acceptance/apply audit records.
- Explicit app-service acceptance and idempotent replay.
- VFS-mediated copy/hardlink/symlink apply primitives.
- Server-side promotion apply with plan revalidation and catalog commit after
  target creation.
- Duplicate relationship persistence from accepted promotion evidence.
- Cleanup-complete and cleanup-pending audit after injected post-storage
  catalog failure.
- Dedicated NFO sidecar follow-on for import/export Library File Writes.

## Goals

- [x] Add durable promotion acceptance/apply audit records separate from preview
  DTOs.
- [x] Revalidate promotion plan facts before any storage mutation.
- [x] Apply copy/hardlink/symlink through VFS/storage backends only.
- [x] Commit Media Source and Source Duplicate Relationship state only after the
  target locator is durable.
- [x] Record cleanup-complete or cleanup-pending evidence after partial failure.
- [x] Keep NFO import/export side effects explicit and split to
  `nfo-sidecar-promotion-apply` instead of hiding them in Managed Import
  promotion.

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
- [nfo-sidecar-promotion-apply](../nfo-sidecar-promotion-apply/README.md)
- [post-rpd-product-hardening](../post-rpd-product-hardening/README.md)
- [addon-library-file-write-policy](../addon-library-file-write-policy/README.md)
- [managed-artwork-fetch-artifact-storage](../managed-artwork-fetch-artifact-storage/README.md)
