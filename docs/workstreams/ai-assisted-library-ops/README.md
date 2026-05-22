# AI Assisted Library Ops

Status: Complete
Last updated: 2026-05-22

## Purpose

This workstream opens the post-network AI productization lane. Nako already has
metadata authority, NFO/link/file-write apply boundaries, Managed Import and
watch-folder intake, playback supportability, network access policy, and the M5
external automation/provider foundation. The remaining product risk is letting
AI-like outputs help operators without silently changing canonical metadata,
sidecars, library files, or Public Client API contracts.

The first safe shape is **Generated Artifact** intake and acceptance, not a core
model runtime. AI-like providers may propose title matches, metadata cleanup,
summaries, and recommendations, but those proposals must remain bounded,
redacted, explainable, and non-canonical until a Nako-owned acceptance workflow
applies them.

## Current Decision

This lane is complete. AILO-020 through AILO-040 shipped Generated Artifact
proposal/readiness semantics, Admin-only redacted proposal diagnostics, typed
Admin Web support, and explicit accept/reject planning for metadata-cleanup
proposals without autonomous canonical metadata, sidecar, Managed Import, Media
Source, library-file, Public Client API, or `nako-client-protocol` mutation.

AILO-050 closed the lane and returned routing to
`post-rpd-product-hardening`. Provider-specific AI adapters, local model
runtime, embeddings/vector search, Public Client display, protocol downloaders,
Addon distribution, and deeper metadata-authority apply remain split follow-ons.

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
