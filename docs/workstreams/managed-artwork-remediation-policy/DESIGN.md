# Managed Artwork Remediation Policy Design

Status: Completed
Last updated: 2026-05-19

## Problem

Storage drift diagnostics can now report missing DB-backed files and stray
artifact-root files, but diagnostics do not define which states Taru may safely
modify. Treating every drift finding as removable would be unsafe:

- missing files for Selected Artwork can break public images and need repair,
  not deletion;
- unrecognized files under the artifact root might not be Taru artifacts;
- parseable files for active artifacts might be current data in an unexpected
  path and need manual inspection;
- files for logically deleted or never-committed artifacts are safe cleanup
  candidates only after active DB state is re-checked.

## Target State

- Admin can request a redacted remediation plan.
- The plan distinguishes advisory missing-artifact remediation from actionable
  stray-file cleanup.
- Only parseable, supported, regular artifact files with no active DB artifact
  are eligible for deletion.
- The delete command requires explicit confirmation and re-checks active DB
  artifact state immediately before deleting each file.
- Missing DB-backed artifacts remain advisory and are never automatically
  removed, repaired, or re-ingested in this lane.
- Responses do not expose filenames, local paths, internal storage handles,
  raw source/cache URLs, provider query strings, token material, or hashes.

## Admin Boundary

Dry-run remediation plan:

```text
GET /admin/v1/artwork/artifacts/remediation-plan?limit=50&offset=0&file_scan_limit=500
```

Explicit stray-file cleanup:

```text
POST /admin/v1/artwork/artifacts/remediate-stray-files?confirm=true&file_scan_limit=500
```

`limit` and `offset` apply to the DB-backed artifact page in the dry-run plan.
`file_scan_limit` bounds local artifact-root inventory. The POST command does
not use DB pagination because it acts only on bounded file inventory.

## Remediation Policy

| Finding | Action | Reason |
| --- | --- | --- |
| Missing selected DB-backed artifact file | Advisory: restore or republish Selected Artwork | Public image identity is protected by Selected Artwork. |
| Missing unselected DB-backed artifact file | Advisory: run artifact cleanup or re-ingest | The DB row is active and must not be silently removed by file cleanup. |
| Untracked parseable artifact file with supported extension and no active DB artifact | Eligible for explicit deletion | The file matches Taru artifact layout and active repository state has no owner. |
| Parseable artifact file for active artifact at unexpected path or extension | Blocked/manual inspect | There is an active DB owner; cleanup must not guess. |
| Unsupported extension or unrecognized layout | Blocked/manual inspect | Not enough proof that Taru owns the file. |

## Redaction Boundary

Admin remediation responses never expose:

- filenames;
- local paths;
- `storage_uri`;
- `managed-artwork://...`;
- raw source URLs;
- `source_uri`;
- `cache_uri`;
- Source Locators;
- addon token material;
- provider query strings;
- content hashes;
- file contents.

## Assumptions

| Assumption | Confidence | Evidence | Mitigation |
| --- | --- | --- | --- |
| Parseable supported artifact files with no active DB artifact are safe deletion candidates. | High | Artifact storage writes `{shard}/{artifact_id}.{jpg,png,webp}` and active lookup hides logically deleted artifacts. | Re-check active artifact by parsed ID before deletion. |
| Missing DB-backed artifacts should not be auto-deleted. | High | Selected Artwork publication uses artifact records as retention authority. | Treat missing files as advisory only. |
| Confirmation query is enough explicit intent for the first Admin cleanup command. | Medium | Other Admin commands are explicit POSTs. | Require `confirm=true` in addition to POST. |

## Splits

- Missing-artifact repair or re-ingest belongs to a future repair lane.
- Unpublish/republish Selected Artwork workflows belong to gallery/selection
  management.
- Thumbnail variants belong to `managed-artwork-thumbnail-variants`.
- Durable retry/requeue/cancellation belongs to
  `managed-artwork-ingest-runtime-controls`.

## Closeout Condition

This lane can close when Taru provides a redacted remediation plan and a
confirmed cleanup command that deletes only cleanable untracked artifact files,
with tests proving active/Selected Artwork state is protected and no path or
storage locator leaks.
