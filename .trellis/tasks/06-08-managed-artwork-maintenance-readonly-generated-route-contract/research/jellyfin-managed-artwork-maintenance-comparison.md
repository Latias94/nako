# Jellyfin Managed Artwork Maintenance Comparison

## Reference Findings

- Jellyfin exposes image/artwork behavior through broad item image APIs,
  configured image fetcher options, cache/system storage status, and maintenance
  tasks rather than a Nako-style Managed Artwork artifact graph.
- Jellyfin's operator model makes image/cache pressure visible, but its codebase
  keeps image paths and filesystem-backed image metadata close to the server
  implementation.
- Nako has a different product boundary: **Managed Artwork** is a
  Nako-owned artifact model with **Selected Artwork** retention and
  redaction-safe Admin diagnostics.

## Nako Gap

- Nako already ships read-only Managed Artwork maintenance routes:
  - artifact lifecycle;
  - artifact storage drift;
  - artifact remediation plan.
- These routes are documented and tested server-side, but remain excluded from
  the generated Admin Web contract, so Admin Web cannot reach them through
  `NAKO_ADMIN_ROUTES`.
- Mutating artwork maintenance commands are useful but higher risk; this slice
  should keep them excluded and expose only diagnostics.

## Chosen Slice

- Generate only the three read-only diagnostic routes.
- Add an Admin Web `/artwork/maintenance` page that renders safe operator facts:
  counts, statuses, safe reason/recommendation enums, IDs, byte counts, media
  types, dimensions, timestamps, and dry-run state.
- Keep destructive cleanup, stray-file deletion, ingest processing, requeue,
  accept, and publish operations out of the UI until a separate confirmation and
  policy task is opened.

## Redaction Boundary

The page and data-source projection must omit:

- `storage_uri`;
- `managed-artwork://...`;
- local paths and artifact root;
- raw filenames;
- raw source URLs and provider query strings;
- `source_uri` and `cache_uri`;
- token or credential material;
- content hash values.

## Validation Implications

- API contract tests should prove generated contracts contain the read-only DTOs
  and routes while mutating maintenance routes remain excluded.
- Admin Web tests should inject unsafe extra fields into mock/live-shaped data
  and assert route rendering omits them.
- Focused server route inventory test should prove the route exclusion list did
  not drift.
