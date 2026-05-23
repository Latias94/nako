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

Reference products to keep in mind:

- Jellyfin shows a catalog/repository split with multiple official and
  third-party plugin repositories.
- Home Assistant treats repository URLs and per-repository add-on manifests as
  the source boundary.
- Visual Studio Code splits extension version from host compatibility through
  `engines.vscode` and supports pre-release channels.
- Obsidian splits plugin version from host compatibility through
  `minAppVersion` and `versions.json`.

Addon Task runtime is a separate follow-on from this lane. The catalog may
list declared task capabilities, but it should not turn a sample declaration
such as `bulk-metadata-scrape` into a contract for host execution, progress, or
results until that runtime lane exists.

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
