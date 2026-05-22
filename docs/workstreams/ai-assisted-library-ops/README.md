# AI Assisted Library Ops

Status: Active
Last updated: 2026-05-22

## Purpose

This workstream opens the post-network AI productization lane. Taru already has
metadata authority, NFO/link/file-write apply boundaries, Managed Import and
watch-folder intake, playback supportability, network access policy, and the M5
external automation/provider foundation. The remaining product risk is letting
AI-like outputs help operators without silently changing canonical metadata,
sidecars, library files, or Public Client API contracts.

The first safe shape is **Generated Artifact** intake and acceptance, not a core
model runtime. AI-like providers may propose title matches, metadata cleanup,
summaries, and recommendations, but those proposals must remain bounded,
redacted, explainable, and non-canonical until a Taru-owned acceptance workflow
applies them.

## Current Decision

AILO-010 opened the lane after Network Access Boundary closeout. AILO-020 then
deepened existing Automation Artifacts into a backend Generated Artifact
proposal queue with stable target, provenance, payload summary, confidence, and
readiness semantics. The next executable slice is AILO-030: expose Admin-only
proposal diagnostics and typed Admin Web support without adding a local model
runtime, autonomous writes, vector database, Addon distribution, downloader
protocols, or Public Client API churn.

## Authoritative Docs

- [Design](DESIGN.md)
- [TODO](TODO.md)
- [Milestones](MILESTONES.md)
- [Evidence and gates](EVIDENCE_AND_GATES.md)
- [Handoff](HANDOFF.md)

## Related Workstreams

- [post-rpd-product-hardening](../post-rpd-product-hardening/README.md)
- [addons-automation](../addons-automation/README.md)
- [metadata-provider-breadth](../metadata-provider-breadth/README.md)
- [nfo-sidecar-promotion-apply](../nfo-sidecar-promotion-apply/README.md)
- [downloads-watch-folder-intake](../downloads-watch-folder-intake/README.md)
- [network-access-boundary](../network-access-boundary/README.md)

## Boundary

This lane owns AI-assisted operator proposals and generated artifact acceptance
semantics. It does not own local model execution, embeddings/vector search,
native plugin/runtime distribution, direct canonical metadata mutation, direct
NFO sidecar writes, direct Media Library writes, downloader protocol adapters,
remote network exposure, or Public Client API shape changes.
