# Post-RPD Product Hardening

## Status

Complete roadmap umbrella. Wave 1 `metadata-provider-breadth` is complete;
`nfo-link-authority`, `managed-import-staging`,
`link-apply-and-import-promotion`, `nfo-sidecar-promotion-apply`,
`playback-transcode-ops-hardening`, `downloads-watch-folder-intake`,
`network-access-boundary`, `ai-assisted-library-ops`, and
`addon-runtime-and-distribution` are complete.

This lane coordinates the post-packaging productization wave after
`release-packaging-and-distribution`. It is not an implementation lane by
itself. It records ordering, dependencies, and split criteria for the next
fearless refactor workstreams so Nako can grow toward a real self-hosted media
library product without collapsing metadata, NFO, playback, import, network,
AI, and addon concerns into one oversized change.

## Purpose

RPD made Nako packageable and diagnosable for self-hosted operators. The next
product risk is whether Nako can safely manage a real library:

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

All planned post-RPD mainline lanes are now represented by dedicated
workstreams and have either completed or split their remaining scope into
explicit follow-ons. Addon Runtime / Distribution closed after package/install
descriptor validation, redacted install-guide preview, Admin-only runtime
readiness diagnostics, declared task/event routing, and Addon Generated
Artifact / acquisition-intake handoff were proven without Addon Manager
automation, process supervision, direct library writes, Public Client protocol
churn, or hidden schedulers.

Any future work should open a focused follow-on lane instead of reopening this
umbrella.

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
