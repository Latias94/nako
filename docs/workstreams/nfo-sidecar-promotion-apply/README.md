# NFO Sidecar Promotion Apply

## Status

Complete follow-on lane split from
`link-apply-and-import-promotion` LAIP-070.

This lane designs and implements accepted NFO sidecar import/export mutation as
an explicit **Library File Write** and metadata-authority workflow. It must not
be hidden inside Managed Import promotion.

Closeout decision: the core sidecar apply boundary is complete. Admin API,
Public Client API, UI, Addon side-effect exposure, and download/watch-folder
automation are follow-on lanes that must consume this accepted apply boundary
instead of writing sidecars directly.

## Purpose

Taru already has NFO parsing/export, round-trip preservation, safer storage
writes, backup policy, backup retention diagnostics, non-mutating NFO authority
preview, and promotion apply safety. The missing product boundary is the actual
operator-confirmed apply step:

- export canonical metadata to a sidecar without destructive rewrite;
- import local NFO authority into canonical metadata and field locks;
- confirm provisional hierarchy when NFO evidence is accepted;
- record every outcome with redacted, replay-safe audit evidence.

## Authoritative Docs

- [Design](DESIGN.md)
- [TODO](TODO.md)
- [Milestones](MILESTONES.md)
- [Evidence and gates](EVIDENCE_AND_GATES.md)
- [Handoff](HANDOFF.md)

## Related Workstreams

- [link-apply-and-import-promotion](../link-apply-and-import-promotion/README.md)
- [nfo-link-authority](../nfo-link-authority/README.md)
- [nfo-round-trip-preservation](../nfo-round-trip-preservation/README.md)
- [nfo-storage-write-policy](../nfo-storage-write-policy/README.md)
- [nfo-sidecar-backup-policy](../nfo-sidecar-backup-policy/README.md)
- [nfo-backup-retention-diagnostics](../nfo-backup-retention-diagnostics/README.md)
- [addon-library-file-write-policy](../addon-library-file-write-policy/README.md)
