# Managed Import Staging — Handoff

Status: Active
Last updated: 2026-05-21

## Current State

MIS-020 and MIS-030 are implemented. Taru now has a durable Managed Import
Artifact domain and DB contract that is intentionally separate from low-level VFS
staging manifests. Records can reference a staging manifest, but they own target
library intent, source kind, import diagnostics, and acceptance/planning state.
The server app service can create/list redacted diagnostics without fetching
external bytes or writing media-library files.

## Active Task

- Task ID: MIS-040
- Owner: codex
- Files: `taru-core`, `taru-server`
- Validation: focused planning tests proving no copy/move/link/delete library
  files and explicit blocked reasons
- Status: READY
- Evidence: MIS-020 and MIS-030 gates are recorded in `EVIDENCE_AND_GATES.md`

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

## Blockers

- None for MIS-040.

## Next Recommended Action

- Execute MIS-040 with TDD: add a non-mutating promotion plan preview that
  explains destination, duplicate/link hints, NFO/provider identity hints, and
  blocked reasons.
