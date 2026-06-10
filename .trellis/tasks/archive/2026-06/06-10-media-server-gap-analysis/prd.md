# brainstorm: compare nako against mature self-hosted media servers

## Goal

Compare Nako against mature self-hosted media servers, especially Jellyfin and Plex, to identify product and architecture gaps for a self-hosted media library/server roadmap.

## What I already know

* Nako is a Rust workspace for a self-hosted media server backend.
* Jellyfin source is available locally under `repo-ref/jellyfin` as GPL reference material.
* Plex is proprietary, so comparison must rely on public product behavior, documentation, and widely known user-facing capabilities rather than source inspection.
* The user wants broad subagent-assisted comparison, not code changes in this turn.
* Nako domain language distinguishes Media Library, Media Source, Media Item, Local Inference, Canonical Metadata, Addon, Playback Runtime, Public Client API, Admin API, User Playback State, and related terms.

## Assumptions

* This task should produce an architectural gap report and an implementation roadmap, not immediate implementation.
* Jellyfin is reference material only; do not copy implementation, schema, migrations, comments, tests, generated files, or assets.
* Recommendations should use Nako domain vocabulary and respect existing ADR decisions unless real friction warrants reopening one.

## Open Questions

* Which gap should become the first implementation workstream after the report is reviewed?

## Requirements

* Compare Nako's current architecture and feature surface against mature self-hosted multimedia server expectations.
* Inspect Jellyfin locally for architecture and capability lessons without copying source.
* Include Plex as a product/UX/operation benchmark from public knowledge and current web documentation where useful.
* Identify Nako gaps by product capability, architecture Module depth, testability, operability, and extensibility.
* Produce a visual architecture review report and a durable plan document.

## Acceptance Criteria

* [x] Findings cite Nako repo paths and relevant ADR/spec sources.
* [x] Jellyfin comparison is based on local reference inspection and clearly separates observed architecture patterns from Nako recommendations.
* [x] Plex comparison avoids source claims and uses public/product-level evidence only.
* [x] Architecture candidates use Module, Interface, Implementation, Depth, Seam, Adapter, Leverage, and Locality language.
* [x] Final output includes a prioritized recommendation for what to tackle first.

## Definition of Done

* Task research notes are written under this task directory.
* A temporary HTML architecture review report is generated and opened.
* A planning artifact is written under `docs/plans/`.
* No code implementation is performed.

## Out of Scope

* Jellyfin Plugin Compatibility.
* Copying GPL implementation details into Nako.
* Implementing any recommended change during this research turn.

## Technical Notes

* Use `CONTEXT.md` domain vocabulary.
* Check relevant ADRs before proposing architectural changes.
* Prefer focused repo inspection and subagent research over large inline raw source dumps.

## Research References

* [`research/jellyfin-reference.md`](research/jellyfin-reference.md) — local Jellyfin reference architecture and capability observations.
* [`research/product-benchmark.md`](research/product-benchmark.md) — Plex/Jellyfin public product capability benchmark and current source URLs.
* [`research/adr-spec-constraints.md`](research/adr-spec-constraints.md) — ADR/spec constraints, conflicts, and open gaps.
* [`research/nako-current-state.md`](research/nako-current-state.md) — Nako current implementation state and gap map.

## Deliverables

* Temporary HTML architecture report: generated outside the repo in the OS temp directory.
* Durable plan: `docs/plans/2026-06-10-001-feat-media-server-maturity-roadmap-plan.md`.
