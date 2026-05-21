# Managed Import Staging

Status: Complete

This workstream is the next post-RPD mainline after `nfo-link-authority`. It
turns downloads, watch-folder candidates, and Addon-proposed artifacts into a
Taru-owned quarantine, validation, and explicit promotion workflow before any
library file mutation occurs.

The lane is intentionally not a generic downloader. Acquisition protocols such
as torrent, Usenet, browser scraping, or Addon external fetch remain outside the
first slice. The first product boundary is **Managed Import Staging**: Taru can
record a proposed artifact, stage it outside the media library, inspect it,
compute duplicate/link/NFO/metadata diagnostics, and produce a promotion plan
that an operator or later acceptance workflow can approve.

## Goals

- Add a durable import staging vocabulary separate from playback/probe staging.
- Keep staged import bytes outside configured media library roots.
- Produce explicit promotion plans before library writes.
- Reuse provider matching, NFO authority preview, and Source Duplicate
  Relationship evidence when explaining a staged artifact.
- Preserve rollback, cleanup, and redacted audit boundaries before actual
  hardlink/symlink or file-copy apply.

## Non-Goals

- No torrent/Usenet/client downloader implementation in the first slice.
- No automatic library writes without promotion confirmation.
- No direct Addon path writes or Addon-owned import state.
- No public client API contract until Admin/operator diagnostics are stable.
- No link apply until promotion, rollback, cleanup, and audit semantics are
  proven.

## Authoritative Docs

- [Design](DESIGN.md)
- [TODO](TODO.md)
- [Milestones](MILESTONES.md)
- [Evidence and gates](EVIDENCE_AND_GATES.md)
- [Handoff](HANDOFF.md)

## Related Workstreams

- [post-rpd-product-hardening](../post-rpd-product-hardening/README.md)
- [metadata-provider-breadth](../metadata-provider-breadth/README.md)
- [nfo-link-authority](../nfo-link-authority/README.md)
- [managed-artwork-fetch-artifact-storage](../managed-artwork-fetch-artifact-storage/README.md)
- [addon-library-file-write-policy](../addon-library-file-write-policy/README.md)
- [link-apply-and-import-promotion](../link-apply-and-import-promotion/README.md)
