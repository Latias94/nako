# Post-RPD Product Hardening

## Status

Active roadmap umbrella. Wave 1 `metadata-provider-breadth` is complete;
`link-apply-and-import-promotion` is complete; `nfo-sidecar-promotion-apply`
is complete. The next recommended mainline lane is Playback/Transcode Ops
Hardening.

This lane coordinates the post-packaging productization wave after
`release-packaging-and-distribution`. It is not an implementation lane by
itself. It records ordering, dependencies, and split criteria for the next
fearless refactor workstreams so Taru can grow toward a real self-hosted media
library product without collapsing metadata, NFO, playback, import, network,
AI, and addon concerns into one oversized change.

## Purpose

RPD made Taru packageable and diagnosable for self-hosted operators. The next
product risk is whether Taru can safely manage a real library:

- explainable TMDB, Douban, and Bangumi metadata matching;
- non-destructive NFO and link authority;
- operator-visible playback/transcode diagnostics;
- safe managed import staging before any download/acquisition breadth;
- remote access boundaries that do not weaken auth or deployment safety;
- AI assistance through generated artifacts and acceptance workflows;
- addon runtime/distribution only after core side-effect boundaries are stable.

## Roadmap Order

1. `metadata-provider-breadth`
2. `nfo-link-authority`
3. `managed-import-staging`
4. `link-apply-and-import-promotion`
5. `nfo-sidecar-promotion-apply`
6. `playback-transcode-ops-hardening`
7. `network-access-boundary`
8. `ai-assisted-library-ops`
9. `addon-runtime-and-distribution`

## Current Decision

After NFO sidecar apply proved accepted import/export, VFS write/restore,
local authority, rollback/repair, and redacted audit boundaries, PRPH-090
re-scores the roadmap. Playback/Transcode Ops Hardening is the next safest
mainline lane because local library mutation boundaries are now proven and
playback confidence is the highest day-to-day operator risk.

## Authoritative Docs

- [Design](DESIGN.md)
- [TODO](TODO.md)
- [Milestones](MILESTONES.md)
- [Evidence and gates](EVIDENCE_AND_GATES.md)
- [Handoff](HANDOFF.md)

## Related Workstreams

- [release-packaging-and-distribution](../release-packaging-and-distribution/README.md)
- [metadata-provider-breadth](../metadata-provider-breadth/README.md)
- [nfo-link-authority](../nfo-link-authority/README.md)
- [managed-import-staging](../managed-import-staging/README.md)
- [link-apply-and-import-promotion](../link-apply-and-import-promotion/README.md)
- [nfo-sidecar-promotion-apply](../nfo-sidecar-promotion-apply/README.md)
- [metadata-catalog](../metadata-catalog/README.md)
- [transcode-runtime](../transcode-runtime/README.md)
- [nfo-round-trip-preservation](../nfo-round-trip-preservation/README.md)
- [nfo-storage-write-policy](../nfo-storage-write-policy/README.md)
- [access-boundary-auth](../access-boundary-auth/README.md)
- [addons-automation](../addons-automation/README.md)
