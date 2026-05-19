# Managed Artwork Artifact Lifecycle Cleanup Milestones

Status: Completed
Last updated: 2026-05-19

## M0 - Open Lane

Exit criteria:

- Workstream docs exist.
- Follow-on scope is split from thumbnails, ingest runtime controls, and
  gallery/candidate management.
- Selected Artwork retention is named as the lifecycle protection boundary.

Status: Done.

## M1 - Lifecycle Diagnostics Dry Run

Exit criteria:

- Admin route lists Managed Artwork Artifact lifecycle state.
- Cleanup candidates are artifacts with zero Selected Artwork references.
- Protected artifacts are reported and not marked eligible.
- Summary byte estimates use artifact metadata and tolerate missing byte
  lengths.
- Response redacts storage URI, local path, raw source URL, cache URI, and
  content hash values.
- No deletion behavior exists in this milestone.

Status: Done.

## M2 - Protected Cleanup Command

Exit criteria:

- Admin cleanup command requires explicit invocation.
- Eligibility is re-checked at deletion time.
- Selected Artwork references prevent deletion through both application logic
  and database constraints.
- Cleanup report is redacted and matches dry-run semantics.
- Local artifact file cleanup is best-effort and reported through redacted
  counts.

Status: Done.

## M3 - Drift Strategy

Exit criteria:

- Missing DB-backed files and stray artifact-root files are either implemented
  through a safe inventory port or split into a follow-on.
- Diagnostics expose counts/status codes only, not local paths.

Status: Split to `managed-artwork-artifact-store-drift-inventory`.

## M4 - Closeout

Exit criteria:

- Focused tests and relevant workspace checks pass.
- Workstream docs record evidence and remaining splits.
- No public client protocol leak or hidden cleanup dependency remains.

Status: Done.
