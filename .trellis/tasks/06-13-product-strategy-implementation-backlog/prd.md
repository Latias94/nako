# plan: Product Strategy Implementation Backlog

## Goal

Turn the competitive research and product positioning work into an executable
backlog that can guide the next Trellis tasks without reframing Nako as a Rust
Jellyfin clone.

The backlog should connect product strategy to implementation order: official
Addon catalog first, then suite packaging and smoke coverage, then remote
access documentation, migration/interop research, ecosystem integrations, and
visible client browse/play polish.

## What I Already Know

- Nako should position itself as an auditable, self-hosted, extensible media
  server backend and control plane.
- Existing roadmap work already covers Product-Operator M1, large-library
  reliability, playback maturity, metadata governance, and Addon ecosystem
  maturity.
- Existing media-server maturity planning already lists broad engineering
  units for operator readiness, playback capability profiles, intake, API
  scale, access policy, realtime/offline split, and Addon lifecycle.
- The new work should not duplicate that engineering maturity plan. It should
  translate the latest competitive research into product-strategy execution
  slices.

## Requirements

- Add a durable backlog document under `docs/plans/`.
- Link the backlog from the documentation index.
- Keep the backlog aligned with:
  - `docs/plans/PRODUCT_POSITIONING.md`
  - `docs/plans/MEDIA_SERVER_PARITY_GAP_MATRIX.md`
  - `docs/plans/ADDON_ECOSYSTEM_STRATEGY.md`
  - `docs/plans/2026-06-10-001-feat-media-server-maturity-roadmap-plan.md`
  - `docs/ROADMAP.md`
- Express work as vertical slices that can become Trellis tasks.
- Preserve the non-goals from the research:
  - no Jellyfin Plugin Compatibility;
  - no native in-process plugin ABI;
  - no Plex-style central account dependency;
  - no first-party traffic relay in the early phase;
  - no copied reference code from Jellyfin or other GPL references.

## Acceptance Criteria

- [ ] The backlog names a recommended first implementation slice.
- [ ] Each slice has a goal, type, dependencies, deliverables, acceptance
      criteria, and non-goals.
- [ ] The backlog distinguishes product-strategy execution from the existing
      media-server maturity engineering plan.
- [ ] `docs/README.md` links to the new backlog.
- [ ] `docs/ROADMAP.md` points M5/Addons and product planning readers toward
      the new backlog without becoming a duplicate task list.
- [ ] The Trellis task records scope and references.

## Definition of Done

- Documentation changes are committed.
- No Rust, TypeScript, schema, generated contracts, or runtime behavior change.
- `git diff --check` passes for the changed docs.

## Technical Approach

Write a strategy-to-execution backlog that acts as a bridge between research
and future tasks. Use issue-like vertical slices, but do not publish external
issues from this task.

## Out of Scope

- Implementing any runtime feature.
- Creating external issue-tracker tickets.
- Updating Addon manifests or official Addon repository code.
- Rewriting M1-M5 roadmap structure.
- Performing new external competitive research.

## Technical Notes

- Research source: `docs/research/nako-product-competitive-analysis/`
- Product strategy sources:
  - `docs/plans/PRODUCT_POSITIONING.md`
  - `docs/plans/MEDIA_SERVER_PARITY_GAP_MATRIX.md`
  - `docs/plans/ADDON_ECOSYSTEM_STRATEGY.md`
- Existing engineering maturity plan:
  - `docs/plans/2026-06-10-001-feat-media-server-maturity-roadmap-plan.md`
- Current roadmap:
  - `docs/ROADMAP.md`
