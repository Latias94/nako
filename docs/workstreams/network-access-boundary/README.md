# Network Access Boundary

Status: Active
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

NAB-010 opened the lane. NAB-020 is complete: Taru now has a network access
policy/readiness domain and config validation surface without runtime tunnel
dialing or Public Client API churn.

NAB-030 is complete: the HTTP boundary now enforces configured browser origins
on protected routes, preserves bearer-auth precedence, keeps `/health` public,
handles allowed CORS preflight requests, and trusts forwarded scheme/host only
when proxy headers and trusted proxy source policy match. The next executable
task is NAB-040 Admin-only network readiness diagnostics.

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
