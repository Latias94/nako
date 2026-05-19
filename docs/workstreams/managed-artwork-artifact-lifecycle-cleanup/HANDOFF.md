# Managed Artwork Artifact Lifecycle Cleanup Handoff

Status: Active
Last updated: 2026-05-19

## Current State

This lane is open and `MAALC-030` is complete.

The implementation adds a redacted Admin diagnostics and cleanup boundary for
Managed Artwork Artifact lifecycle state:

- classify artifacts with zero Selected Artwork references as cleanup
  candidates;
- classify artifacts with one or more Selected Artwork references as protected;
- expose safe IDs, kind, byte/dimension/media facts, and counts only;
- mark eligible cleanup candidates logically deleted through
  `managed_artwork_artifacts.deleted_at`;
- hide logically deleted artifacts from active artifact lookup and lifecycle
  diagnostics;
- best-effort delete local artifact files after repository cleanup and report
  only redacted counts;
- omit `storage_uri`, `managed-artwork://...`, local paths, raw source URLs,
  `source_uri`, `cache_uri`, Source Locators, addon token material, provider
  query strings, and content hashes.

## Next Step

Decide whether `MAALC-040` should remain in this lane or split into a dedicated
artifact-store drift inventory lane. The next design question is root inventory:
missing DB-backed files and stray files under the artifact root require a safe
storage inventory port that never reports local paths.

## Blockers

None known.

## Follow-Ons Outside This Lane

- `managed-artwork-thumbnail-variants`
- `managed-artwork-ingest-runtime-controls`
- `managed-artwork-gallery-candidate-management`
