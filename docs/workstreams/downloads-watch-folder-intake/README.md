# Downloads / Watch-Folder Intake

Status: Complete
Last updated: 2026-05-22

## Purpose

This workstream opens the first acquisition breadth lane after metadata,
NFO/link, Managed Import, accepted promotion apply, NFO sidecar apply, and
playback supportability are proven.

The correct first product shape is not a built-in downloader. Taru should first
learn how to discover or accept acquisition candidates, represent them as
Taru-owned staged artifacts, explain their risk, and hand them off to existing
promotion/apply workflows without writing directly into a Media Library.

## Current Decision

DWI-010 opened the lane as the next mainline child of
`post-rpd-product-hardening`.

DWI-020 is complete. Taru now has durable acquisition intake candidate domain
records and backend-neutral persistence for watch-folder/operator candidates.

DWI-030 is complete. Taru now has an app-service boundary that records/lists
redacted intake candidates and accepts candidates into existing or new Managed
Import artifacts without promotion apply, Media Source creation, or library file
mutation.

DWI-040 is complete. Taru now discovers watch-folder candidates through
storage/VFS list/stat boundaries, classifies ready/incomplete/unsupported
entries, and writes idempotent intake records without trusting raw host paths or
creating Managed Import artifacts, Media Sources, promotion applies, or library
file writes.

DWI-050 is complete. Taru now exposes Admin-only, redacted acquisition intake
candidate diagnostics and watch-folder discovery controls through Admin API v1,
with synchronized Admin TypeScript contract/client support and no Public Client
API or `taru-client-protocol` changes.

DWI-060 is complete. Final closeout gates passed, follow-ons were split rather
than hidden in this lane, and the next product lane decision returned to the
post-RPD umbrella.

## Authoritative Docs

- [Design](DESIGN.md)
- [TODO](TODO.md)
- [Milestones](MILESTONES.md)
- [Evidence and gates](EVIDENCE_AND_GATES.md)
- [Handoff](HANDOFF.md)

## Related Workstreams

- [post-rpd-product-hardening](../post-rpd-product-hardening/README.md)
- [managed-import-staging](../managed-import-staging/README.md)
- [link-apply-and-import-promotion](../link-apply-and-import-promotion/README.md)
- [nfo-sidecar-promotion-apply](../nfo-sidecar-promotion-apply/README.md)
- [playback-transcode-ops-hardening](../playback-transcode-ops-hardening/README.md)
- [storage-vfs](../storage-vfs/README.md)
- [nfo-link-authority](../nfo-link-authority/README.md)

## Boundary

This lane owns acquisition intake and watch-folder candidate discovery. It does
not own torrent/Usenet/download-client protocols, remote access/tunnel runtime,
AI generation, Addon runtime/distribution, automatic promotion apply, NFO
sidecar mutation shortcuts, or direct filesystem writes into media libraries.
