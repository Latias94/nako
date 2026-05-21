# Managed Import Staging — Handoff

Status: Complete
Last updated: 2026-05-21

## Current State

MIS-020 through MIS-060 are complete. Taru now has a durable Managed Import
Artifact domain and DB contract that is intentionally separate from low-level VFS
staging manifests. Records can reference a staging manifest, but they own target
library intent, source kind, import diagnostics, and acceptance/planning state.
The server app service can create/list redacted diagnostics without fetching
external bytes or writing media-library files. It can also produce a
non-mutating promotion preview that explains the destination Source Locator,
copy/move/link dry-run options, duplicate hints, NFO sidecar authority hints,
provider identity review hints, and explicit blocked reasons.

Actual promotion apply is split to
`docs/workstreams/link-apply-and-import-promotion` because it is the first
boundary that may mutate a Media Library root and catalog state.

## Final State

- Task ID: MIS-060
- Owner: planner
- Files: `docs/workstreams/managed-import-staging`
- Validation: evidence gates are fresh; split decision is explicit; parent
  umbrella and index route to the follow-on
- Status: DONE
- Evidence: MIS-020, MIS-030, MIS-040, MIS-050, and MIS-060 entries are
  recorded in `EVIDENCE_AND_GATES.md`

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
- Promotion apply is not part of Managed Import Staging. It is split to
  `link-apply-and-import-promotion` with explicit operator confirmation, plan
  revalidation, durable audit, rollback/cleanup, VFS-only mutation, catalog
  consistency, and NFO boundary requirements.

## Blockers

- None.

## Follow-Ons

- `link-apply-and-import-promotion`: implement actual promotion apply only
  after durable acceptance/audit, plan revalidation, storage mutation
  primitives, rollback/cleanup, and catalog consistency gates are proven.
- Admin/operator API: expose Managed Import diagnostics and promotion previews
  after the app-internal DTOs stabilize.
- Downloader/watch-folder acquisition: remain downstream of Managed Import and
  apply semantics; do not introduce protocol-specific acquisition into this
  closed staging lane.

## Next Recommended Action

- Continue with `link-apply-and-import-promotion` LAIP-020, the durable
  promotion acceptance and audit model.
