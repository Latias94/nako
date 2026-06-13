# feat: Official Addon Catalog Minimal Loop

## Goal

Build the first visible product loop for Nako's official Addon ecosystem: a
durable, generated catalog of official Addons that operators can inspect, and
that release gates can validate, without introducing host-managed addon
lifecycle.

## What I Already Know

- Nako's Addon strategy is sidecar-first and explicitly rejects an in-process
  plugin ABI.
- The product strategy backlog recommends the official catalog as the first
  implementation slice.
- The official Addon ecosystem already has concrete manifests and helper
  crates, so the catalog should be generated from authoritative addon facts
  rather than maintained as a separate handwritten list.
- The first slice should stay small: catalog discovery, compatibility,
  install-guide references, and smoke status.

## Assumptions

- The official addon facts can be sourced from the repository's existing
  official addon catalog crate and/or the addon repository manifests.
- A generated or validated markdown/JSON catalog artifact is acceptable for
  the first slice.
- The first slice does not need one-click installation or addon lifecycle
  management.

## Open Questions

- Which source should be treated as the initial authority for addon facts:
  the local catalog crate, the addon repo manifests, or both?
- What output format should the catalog use for the first slice?
- Which smoke status fields are realistic for the first version?

## Requirements

- Create a durable official Addon catalog artifact in the repo.
- Include addon id, version, protocol version, compatible Nako version range,
  resource/task/event declarations, scopes, install reference, trust tier, and
  smoke status.
- Link the catalog from the documentation entry points.
- Keep the catalog generation/validation path deterministic.
- Preserve the product boundary: discovery and compatibility only, not addon
  manager behavior.

## Acceptance Criteria

- [ ] Every official Addon has one catalog entry or descriptor.
- [ ] The catalog can be validated locally or in CI.
- [ ] The catalog distinguishes Addon Version, Addon Protocol Version, and
      compatible Nako version.
- [ ] The catalog shows trust tier and smoke status without leaking secrets.
- [ ] Docs point operators to the catalog.

## Definition of Done

- Tests or validation scripts added/updated where needed.
- Docs/indexes updated.
- No runtime addon lifecycle management is added.

## Technical Notes

- Product strategy backlog:
  `docs/plans/PRODUCT_STRATEGY_IMPLEMENTATION_BACKLOG.md`
- Addon strategy:
  `docs/plans/ADDON_ECOSYSTEM_STRATEGY.md`
- Existing official addon catalog crate:
  `crates/nako-official-addon-catalog/`
- Official addon repository:
  `../nako-official-addons/`
