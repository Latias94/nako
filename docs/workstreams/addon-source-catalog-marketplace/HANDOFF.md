# Addon Source Catalog And Marketplace - Handoff

Status: Active
Last updated: 2026-05-23

## Current State

The Addon Manager lifecycle lane is complete. Nako now exposes the first
manager-owned registry/plan slot and the official addon alpha smoke remains
repeatable.

This lane exists to decide how addon sources become discoverable catalog or
marketplace entries before an operator confirms a lifecycle plan. It should not
collapse into package signing, provider breadth, or direct process/container
supervision.

## Next Task

Continue with ASCM-010.

Goal: freeze the addon source catalog / marketplace boundary, non-goals, and
first discovery slice.

Suggested first steps:

1. Re-read ADR 0020 and the completed Addon Manager closeout for the existing
   manager boundary.
2. Decide whether source listing, browse metadata, and resolution belong in one
   lane or should split.
3. Keep package signing, provider breadth, and process supervision out of the
   first slice.
4. Record the split/follow-on boundaries before implementation.

## Known Risks

- A catalog lane can accidentally absorb package signing or process
  supervision if the first slice is not narrow.
- The existing manager-plan and official addon smoke must stay valid while the
  discovery lane evolves.
- Resolution and browse metadata may need their own test fixtures if the lane
  grows beyond the first catalog slice.
