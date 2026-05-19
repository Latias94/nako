# Managed Artwork Artifact Store Drift Inventory Milestones

Status: Completed
Last updated: 2026-05-19

## M0 - Open Split Lane

Exit criteria:

- Workstream docs exist.
- Lifecycle cleanup no longer owns artifact-root drift scanning.
- Non-goals exclude deletion, repair, thumbnails, runtime controls, and gallery
  management.

Status: Done.

## M1 - Storage Drift Diagnostics

Exit criteria:

- Admin route returns read-only storage drift diagnostics.
- Active DB-backed artifacts are checked for missing/unresolvable expected
  files.
- Artifact-root inventory is bounded and reports truncation.
- Stray files are classified without exposing filenames or local paths.
- Response redacts storage URI, local path, raw source URL, cache URI, provider
  query strings, addon token material, and content hash values.

Status: Done.

## M2 - Closeout Or Remediation Split

Exit criteria:

- Focused tests and relevant workspace checks pass.
- Workstream docs record evidence and remaining splits.
- Any deletion/repair/re-ingest behavior is explicitly split.

Status: Done.
