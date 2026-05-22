# Network Access Boundary

Status: Complete
Last updated: 2026-05-22

## Purpose

This workstream opens the post-DWI remote access lane. Taru already has inbound
bearer authentication, packaged self-hosted deployment docs, playback
supportability, and acquisition intake boundaries. The next product-hardening
risk is letting real clients reach Taru through reverse proxies, tunnels, VPNs,
or private networks without weakening auth, origin policy, path redaction, or
library mutation boundaries.

The first safe shape is not built-in NAT traversal. Taru should first define a
server-owned network access policy and diagnostics model: configured external
base URLs, trusted proxy/header handling, tunnel-provider registration and
readiness, CORS/origin constraints, and Admin-only redacted diagnostics.

## Current Decision

This lane is complete. NAB-020 through NAB-040 shipped a network access
policy/readiness domain, request-time HTTP boundary enforcement, and
Admin-only redacted network diagnostics without starting a built-in NAT
traversal runtime or changing Public Client API / `taru-client-protocol`.

NAB-050 closed the lane and returned routing to
`post-rpd-product-hardening`. Concrete tunnel runtimes, endpoint discovery,
identity/RBAC, protocol downloader integrations, and Addon runtime/distribution
remain split follow-ons.

## Authoritative Docs

- [Design](DESIGN.md)
- [TODO](TODO.md)
- [Milestones](MILESTONES.md)
- [Evidence and gates](EVIDENCE_AND_GATES.md)
- [Handoff](HANDOFF.md)

## Related Workstreams

- [post-rpd-product-hardening](../post-rpd-product-hardening/README.md)
- [access-boundary-auth](../access-boundary-auth/README.md)
- [release-packaging-and-distribution](../release-packaging-and-distribution/README.md)
- [playback-transcode-ops-hardening](../playback-transcode-ops-hardening/README.md)
- [downloads-watch-folder-intake](../downloads-watch-folder-intake/README.md)
- [public-api-contract](../public-api-contract/README.md)

## Boundary

This lane owns inbound network exposure policy, external endpoint readiness,
trusted proxy/header rules, tunnel-provider abstraction/readiness, and safe
Admin diagnostics. It does not own built-in NAT traversal runtime, relay
servers, TURN/STUN, downloader protocols, AI writes, Addon runtime, multi-user
RBAC, or library file mutation.
