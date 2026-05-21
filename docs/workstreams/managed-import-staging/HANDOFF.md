# Managed Import Staging — Handoff

Status: Active
Last updated: 2026-05-21

## Current State

MIS-020 through MIS-040 are implemented. Taru now has a durable Managed Import
Artifact domain and DB contract that is intentionally separate from low-level VFS
staging manifests. Records can reference a staging manifest, but they own target
library intent, source kind, import diagnostics, and acceptance/planning state.
The server app service can create/list redacted diagnostics without fetching
external bytes or writing media-library files. It can also produce a
non-mutating promotion preview that explains the destination Source Locator,
copy/move/link dry-run options, duplicate hints, NFO sidecar authority hints,
provider identity review hints, and explicit blocked reasons.

## Active Task

- Task ID: MIS-050
- Owner: planner
- Files: `docs/workstreams/managed-import-staging`
- Validation: DESIGN/HANDOFF document rollback, cleanup, audit, and operator
  confirmation requirements
- Status: READY
- Evidence: MIS-020, MIS-030, and MIS-040 gates are recorded in
  `EVIDENCE_AND_GATES.md`

## Decisions

- Do not implement torrent/Usenet/downloader protocols in this lane's first
  slice.
- Do not write, copy, move, link, or delete files in media library roots during
  staging or preview.
- Do not overload `StagingManifestRecord` as the operator-facing import artifact
  model.
- Managed Import records may reference VFS staging manifests, but they own
  product import intent, target library, diagnostics, and acceptance state.
- Promotion apply remains separate until rollback, cleanup, audit, and operator
  confirmation are proven.
- First durable states are represented in core as explicit enum values through
  `planned`; apply states exist for lifecycle completeness but are not exercised
  by MIS-020.
- MIS-030 diagnostics are intentionally app-internal for now; HTTP/Admin API can
  be added later when the diagnostic DTO is stable.
- Diagnostics expose booleans and redacted URI scheme, not raw source URI,
  artifact URI, original file name, intended locator, fingerprint, or raw
  diagnostics JSON.
- MIS-040 promotion preview is read-only. Copy/move operations are represented
  as future-apply options; hardlink/symlink status comes from VFS `plan_link`;
  no storage `write`, `stage`, copy, move, link, delete, Media Source insert, or
  NFO export is performed.
- NFO/provider identity hints are intentionally explanatory, not canonical
  metadata writes. Provider identity remains a review signal until promotion
  apply has an acceptance workflow.

## Blockers

- None for MIS-050.

## Next Recommended Action

- Execute MIS-050: decide whether first promotion apply can safely live in this
  lane, or split to a dedicated `link-apply-and-import-promotion` follow-on with
  rollback, cleanup, audit, operator confirmation, and storage mutation gates.
