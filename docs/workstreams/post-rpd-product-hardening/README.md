# Post-RPD Product Hardening

## Status

Active roadmap umbrella. Wave 1 `metadata-provider-breadth` is complete;
`nfo-link-authority`, `managed-import-staging`,
`link-apply-and-import-promotion`, `nfo-sidecar-promotion-apply`, and
`playback-transcode-ops-hardening` are complete. `downloads-watch-folder-intake`
is open as the current mainline lane.

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
7. `downloads-watch-folder-intake`
8. `network-access-boundary`
9. `ai-assisted-library-ops`
10. `addon-runtime-and-distribution`

## Current Decision

After Playback/Transcode Ops Hardening closed, PRPH-110 re-scored the remaining
roadmap. PRPH-120 opened `downloads-watch-folder-intake` as the next mainline
lane, but only as staged artifact intake that consumes Taru-owned Managed
Import Staging, Link Apply, NFO Sidecar Apply, and playback support evidence.
DWI-050 Admin-only intake diagnostics are complete. The next executable task is
DWI-060 closeout and follow-on split for `downloads-watch-folder-intake`.

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
- [playback-transcode-ops-hardening](../playback-transcode-ops-hardening/README.md)
- [downloads-watch-folder-intake](../downloads-watch-folder-intake/README.md)
- [metadata-catalog](../metadata-catalog/README.md)
- [transcode-runtime](../transcode-runtime/README.md)
- [nfo-round-trip-preservation](../nfo-round-trip-preservation/README.md)
- [nfo-storage-write-policy](../nfo-storage-write-policy/README.md)
- [access-boundary-auth](../access-boundary-auth/README.md)
- [addons-automation](../addons-automation/README.md)
