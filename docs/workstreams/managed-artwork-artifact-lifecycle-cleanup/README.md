# Managed Artwork Artifact Lifecycle Cleanup

Status: Completed
Last updated: 2026-05-19

## Purpose

This lane follows `managed-artwork-public-serving-selection`. Nako can now store
Managed Artwork Artifact bytes, publish Selected Artwork, and serve selected
images through first-party routes. The remaining lifecycle risk is operational:
stored artifacts can become unselected, file and database state can drift, and
future cleanup must never delete artwork still referenced by Selected Artwork.

## Goals

- Define the Managed Artwork Artifact lifecycle boundary around Nako-owned
  artifact records and internal storage.
- Provide a redacted Admin diagnostics and cleanup dry-run view.
- Identify cleanup candidates as artifacts that are not referenced by Selected
  Artwork.
- Preserve Selected Artwork references as a hard retention boundary.
- Provide a protected cleanup command that removes only eligible orphan
  artifacts.

## Non-Goals

- Thumbnail generation or responsive variants.
- Durable ingest retry, requeue, cancellation, or job runtime controls.
- Public gallery/candidate management.
- Public Client API image contract changes.
- Addon-side fetching, addon-owned artifact storage, or Artwork Export.

## Authoritative Docs

- `DESIGN.md`
- `TODO.md`
- `MILESTONES.md`
- `EVIDENCE_AND_GATES.md`
- `HANDOFF.md`
- `WORKSTREAM.json`

## Current Slice

This lane completed the redacted lifecycle diagnostics and protected cleanup
command. Artifact-root drift inventory was split to
`managed-artwork-artifact-store-drift-inventory`.
