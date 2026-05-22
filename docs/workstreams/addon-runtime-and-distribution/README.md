# Addon Runtime And Distribution

Status: Complete
Last updated: 2026-05-22

## Purpose

This workstream opens the post-AI Addon productization lane. Taru already has
HTTP Addon Sidecars, Addon Protocol types, Addon Tokens and grants, Addon Side
Effects, protected metadata/artwork/library-file write paths, Admin Addon
operations, Network Access Boundary readiness, and Generated Artifact
proposal/review semantics.

The remaining product risk is turning manually registered sidecars into a safe
operator distribution story without weakening the extension boundary. The first
safe shape is **distribution and runtime readiness**, not a native plugin ABI:
validate addon packages/manifests, generate install guidance, check sidecar
runtime health, and route declared tasks/events/artifacts through Taru-owned
queues and side-effect APIs.

## Current Decision

This lane is complete. ARD-020 through ARD-050 shipped the Addon Sidecar
package/install descriptor boundary, redacted install-guide preview, Admin-only
runtime readiness diagnostics, declared task/event routing plans, and
Addon-produced Generated Artifact / acquisition-intake handoff.

ARD-060 closes the lane and returns routing to
`post-rpd-product-hardening`. Addon Manager discovery/install/update,
marketplace hosting, package signing trust roots, process/container
supervision, rollback/log streaming, Native Plugin ABI, downloader protocol
adapters, local AI/model runtime, Public Client surfaces, and
`taru-client-protocol` changes remain split follow-ons rather than hidden in
this lane.

## Authoritative Docs

- [Design](DESIGN.md)
- [TODO](TODO.md)
- [Milestones](MILESTONES.md)
- [Evidence and gates](EVIDENCE_AND_GATES.md)
- [Handoff](HANDOFF.md)

## Related Workstreams

- [post-rpd-product-hardening](../post-rpd-product-hardening/README.md)
- [addons-automation](../addons-automation/README.md)
- [addon-architecture-deepening](../addon-architecture-deepening/README.md)
- [admin-addon-operations-mvp](../admin-addon-operations-mvp/README.md)
- [downloads-watch-folder-intake](../downloads-watch-folder-intake/README.md)
- [ai-assisted-library-ops](../ai-assisted-library-ops/README.md)
- [network-access-boundary](../network-access-boundary/README.md)

## Boundary

This lane owns Addon Sidecar package/manifest distribution readiness, install
guidance, runtime health/readiness, declared task/event/artifact routing, and
Admin-only operator diagnostics. It does not own Native Plugin ABI, Jellyfin
plugin compatibility, marketplace hosting, automatic container/process
supervision, package signing trust root, direct filesystem writes, Public
Client API churn, or `taru-client-protocol` changes.
