# NFO Link Authority

Status: Complete

This workstream was the next post-RPD mainline lane after
`metadata-provider-breadth`. It turned Nako's local metadata and file-link
behavior into explicit authority boundaries before managed import/download,
AI suggestions, or addon distribution can depend on library-file mutation.

Existing NFO work already covers round-trip preservation, atomic local writes,
same-directory backups, and backup retention. The remaining risk is broader
local authority: NFO export/import decisions, source duplicate/link evidence,
and future soft/hard link operations must be dry-run, diagnosable, and
rollback-aware before Nako creates or rewrites library files.

Closeout decision: actual hardlink/symlink creation is split to a follow-on
after `managed-import-staging` defines promotion, rollback, cleanup, audit, and
source duplicate confirmation semantics.

## Goals

- Preserve NFO as a first-class local metadata authority.
- Add non-destructive link planning diagnostics before any link mutation.
- Keep soft/hard link behavior in VFS/storage, not in the NFO codec.
- Use **Source Duplicate Relationship** for link/duplicate evidence instead of
  merging sources or items automatically.
- Prepare the safe foundation for later `managed-import-staging`.

## Non-Goals

- No automatic symlink or hardlink creation in the first slice.
- No link deletion, cleanup, or rollback execution until dry-run and apply
  reports are proven.
- No generic downloader or acquisition protocol.
- No provider metadata breadth.
- No direct addon path writes.

## Authoritative Docs

- [Design](DESIGN.md)
- [TODO](TODO.md)
- [Milestones](MILESTONES.md)
- [Evidence and gates](EVIDENCE_AND_GATES.md)
- [Handoff](HANDOFF.md)

## Related Workstreams

- [post-rpd-product-hardening](../post-rpd-product-hardening/README.md)
- [metadata-provider-breadth](../metadata-provider-breadth/README.md)
- [nfo-round-trip-preservation](../nfo-round-trip-preservation/README.md)
- [nfo-storage-write-policy](../nfo-storage-write-policy/README.md)
- [nfo-sidecar-backup-policy](../nfo-sidecar-backup-policy/README.md)
- [nfo-backup-retention-diagnostics](../nfo-backup-retention-diagnostics/README.md)
- [addon-library-file-write-policy](../addon-library-file-write-policy/README.md)
