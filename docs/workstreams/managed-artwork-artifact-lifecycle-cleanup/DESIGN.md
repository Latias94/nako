# Managed Artwork Artifact Lifecycle Cleanup Design

Status: Completed
Last updated: 2026-05-19

## Problem

Managed artwork now has three separate authorities:

- `managed_artwork_artifacts` stores the internal artifact metadata and
  `managed-artwork://...` storage authority.
- `selected_artworks` publishes one artifact as the current Selected Artwork
  for an item and image kind.
- `LocalManagedArtworkArtifactStore` owns bytes below the configured artifact
  root.

The publication lane intentionally deferred cleanup. Without a lifecycle
boundary, unselected artifacts can accumulate, operators cannot inspect storage
growth safely, and a future cleanup command could accidentally remove bytes
used by Selected Artwork.

## Target State

- Admin diagnostics expose a redacted lifecycle snapshot for Managed Artwork
  Artifacts.
- Cleanup candidates are defined by repository state, not by paths:
  `selected_artworks` has no row referencing the artifact.
- Protected artifacts have one or more Selected Artwork references and are not
  eligible for cleanup.
- Dry-run output shows IDs, item/library/kind, selected-reference count,
  cleanup-candidate status, safe media facts, and byte-count estimates.
- Public Client DTOs remain unchanged.
- Admin DTOs never expose `storage_uri`, `managed-artwork://...`, local paths,
  raw source URLs, `source_uri`, `cache_uri`, Source Locators, addon token
  material, provider query strings, or content hashes.

## Architecture Direction

### Lifecycle Read Model

The repository owns lifecycle inventory because it can atomically join
`managed_artwork_artifacts` and `selected_artworks`. The first query returns:

- a summary over all artifacts;
- paginated rows;
- `selected_artwork_count`;
- `cleanup_candidate = selected_artwork_count == 0`.

The first implementation deliberately does not enumerate files under the
artifact root. File-store drift detection is useful, but it needs a separate
storage inventory port so Admin diagnostics do not leak local paths or couple
SQL queries to filesystem layout.

### Admin Boundary

The first Admin route is:

```text
GET /admin/v1/artwork/artifacts/lifecycle?cleanup_candidates_only=false&limit=50&offset=0
```

This is a dry-run diagnostics route. It reports what cleanup would consider,
but it does not delete rows or files.

### Cleanup Boundary

The explicit cleanup command is:

```text
POST /admin/v1/artwork/artifacts/cleanup?limit=50&offset=0
```

It:

- require explicit Admin intent;
- re-check eligibility in the same repository operation used for cleanup;
- mark only artifacts with no Selected Artwork reference as deleted;
- hide logically deleted artifacts from artifact lookup and lifecycle
  diagnostics;
- best-effort remove local artifact bytes after repository cleanup;
- report file cleanup counts without exposing paths;
- avoid returning internal storage locators;
- prefer database guards and `ON DELETE RESTRICT` over trusting application
  filtering alone.

Cleanup is logical at the database layer (`managed_artwork_artifacts.deleted_at`)
so ingest history remains auditable. Physical file removal is best-effort and
reported separately because storage cleanup can fail independently from the
repository state transition.

## Existing Guards

`selected_artworks.artifact_id` references `managed_artwork_artifacts(id)` with
`ON DELETE RESTRICT`. Cleanup preserves that invariant and adds a repository
`NOT EXISTS selected_artworks` guard before marking any artifact deleted.

## Assumptions

| Assumption | Confidence | Evidence | Mitigation |
| --- | --- | --- | --- |
| `selected_artworks` is the public image identity and retention authority. | High | MAPS closeout documents `selected_artworks.id` as public image ID authority. | Keep cleanup candidate selection based on Selected Artwork references. |
| Admin diagnostics can expose artifact IDs safely. | High | Existing Admin publish and ingest responses expose artifact IDs while redacting storage. | Continue redaction tests for storage handles and paths. |
| Filesystem drift detection needs a separate storage inventory port. | Medium | Current local artifact store can write, read selected images, and best-effort delete rollback paths only. | Split drift/orphan-file scanning if implementation needs directory enumeration. |

## Splits

- Thumbnail variants belong to `managed-artwork-thumbnail-variants`.
- Durable retry/requeue/cancellation belongs to
  `managed-artwork-ingest-runtime-controls`.
- Gallery and candidate browsing belongs to
  `managed-artwork-gallery-candidate-management`.
- File-store drift inventory can become a separate lane if it requires artifact
  root enumeration beyond DB-backed cleanup.

## Closeout Condition

This lane can close when Admin diagnostics and cleanup commands can identify,
report, and remove only unselected Managed Artwork Artifacts while protecting
Selected Artwork, preserving redaction, and recording fresh validation evidence.
