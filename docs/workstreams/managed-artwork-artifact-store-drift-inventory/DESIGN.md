# Managed Artwork Artifact Store Drift Inventory Design

Status: Completed
Last updated: 2026-05-19

## Problem

Managed Artwork now has a safe lifecycle cleanup path, but the cleanup path only
operates on DB-backed artifact records. Local storage can still drift:

- an active `managed_artwork_artifacts` row can be present while its expected
  file is missing;
- a file can remain under the artifact root after its DB row has been logically
  deleted or was never committed;
- arbitrary files can appear under the artifact root.

The diagnostics boundary must not turn local filesystem layout into a public
or Admin API contract. Operators need counts and safe IDs, not paths.

## Target State

- Admin can request a read-only storage drift snapshot.
- DB-backed active artifacts are checked through the same storage authority
  rules used for selected image serving.
- Artifact-root file inventory stays inside the local artifact store.
- Stray file diagnostics return category counts and safe optional parsed
  artifact IDs, never filenames or local paths.
- The endpoint is bounded by explicit artifact and file scan limits.
- No deletion, repair, hash calculation, or content validation occurs.

## Admin Boundary

The first Admin route is:

```text
GET /admin/v1/artwork/artifacts/storage-drift?limit=50&offset=0&file_scan_limit=500
```

`limit` and `offset` page DB-backed active artifact checks. `file_scan_limit`
bounds the artifact-root file inventory. The response is a dry-run diagnostics
view and is not a cleanup command.

## Architecture Direction

### DB-Backed Missing Files

The app service asks the repository for the active Managed Artwork Artifact
lifecycle page. For each active artifact in that page, the local artifact store
resolves the expected internal path from:

- `storage_uri == managed-artwork://artifact/{artifact_id}`;
- the artifact ID shard;
- the stored media type extension.

The response can include the artifact ID, library ID, item ID, image kind,
Selected Artwork reference count, cleanup-candidate flag, byte length, media
type, and a redacted issue code. It cannot include the internal storage URI,
path, content hash, source URI, cache URI, provider query string, or token
material.

### Artifact-Root Stray Files

The local artifact store performs bounded non-following directory inventory
under the configured artifact root. It classifies each regular file or
non-directory entry without returning its path:

- parseable Taru artifact file with no active DB artifact;
- parseable Taru artifact file in an unexpected location or extension for its
  active DB artifact;
- unsupported extension;
- unrecognized layout.

The app service may use active DB lookup by parsed artifact ID to avoid false
positive stray reports for active artifacts outside the current artifact page.

### Redaction Boundary

Admin diagnostics never expose:

- `storage_uri`;
- `managed-artwork://...`;
- local paths or filenames;
- raw source URLs;
- `source_uri`;
- `cache_uri`;
- Source Locators;
- addon token material;
- provider query strings;
- content hashes.

## Assumptions

| Assumption | Confidence | Evidence | Mitigation |
| --- | --- | --- | --- |
| Artifact root belongs to Taru and can be scanned with a bounded limit. | High | Fetch/storage lane writes managed bytes below a dedicated artwork artifact root. | Keep scan bounded and report truncation. |
| Active DB artifact lookup is enough to decide whether a parseable file is stray. | High | Lifecycle cleanup hides `deleted_at` rows from active artifact lookup. | Treat files for logically deleted rows as untracked stray files. |
| Parsed artifact IDs from file names are safe to expose to Admin diagnostics. | Medium | Admin APIs already expose artifact IDs. | Expose only parsed UUID IDs, not filenames, paths, or storage URIs. |

## Splits

- Stray file deletion or repair belongs to a future remediation lane.
- Missing DB-backed artifact repair belongs to a future artifact repair or
  re-ingest lane.
- Thumbnail variants belong to `managed-artwork-thumbnail-variants`.
- Durable retry/requeue/cancellation belongs to
  `managed-artwork-ingest-runtime-controls`.
- Gallery and candidate browsing belongs to
  `managed-artwork-gallery-candidate-management`.

## Closeout Condition

This lane can close when Admin storage drift diagnostics identify missing
DB-backed files and stray artifact-root files through bounded, redacted,
read-only evidence, with fresh tests proving no local path or internal storage
locator leaks.
