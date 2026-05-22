# Post-RPD Product Hardening

## Status

Active roadmap umbrella. Wave 1 `metadata-provider-breadth` is complete;
`nfo-link-authority`, `managed-import-staging`,
`link-apply-and-import-promotion`, `nfo-sidecar-promotion-apply`,
`playback-transcode-ops-hardening`, `downloads-watch-folder-intake`,
`network-access-boundary`, and `ai-assisted-library-ops` are complete.
`addon-runtime-and-distribution` is open as the current mainline lane.

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

Network Access Boundary and AI Assisted Library Ops are complete. PRPH-150
closed Network Access Boundary and opened AI Assisted Library Ops. PRPH-160
opened the AI lane, and AILO-050 returned to this umbrella after Generated
Artifact proposal/readiness, Admin diagnostics, and explicit accept/reject
planning were proven without autonomous writes. PRPH-170 now selects Addon
Runtime / Distribution as the current mainline lane. ARD-020 completed the
package/install descriptor and redacted install-guide boundary. The next
executable task is ARD-030 runtime readiness diagnostics.

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
- [network-access-boundary](../network-access-boundary/README.md)
- [ai-assisted-library-ops](../ai-assisted-library-ops/README.md)
- [addon-runtime-and-distribution](../addon-runtime-and-distribution/README.md)
- [metadata-catalog](../metadata-catalog/README.md)
- [transcode-runtime](../transcode-runtime/README.md)
- [nfo-round-trip-preservation](../nfo-round-trip-preservation/README.md)
- [nfo-storage-write-policy](../nfo-storage-write-policy/README.md)
- [access-boundary-auth](../access-boundary-auth/README.md)
- [addons-automation](../addons-automation/README.md)
