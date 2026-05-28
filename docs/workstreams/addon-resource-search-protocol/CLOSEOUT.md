# Addon Resource Search Protocol - Closeout

Status: Closed
Closed: 2026-05-28

## Closeout Claim

The addon resource-search protocol lane is complete. Nako now has a first-class
read-only addon resource for acquisition resource discovery, a typed client
helper, an admin diagnostic host boundary, and an explicit host-owned handoff
from selected resource-search links into acquisition intake candidates.

## Delivered

- `nako-addon-protocol` defines `AddonResource::ResourceSearch` and
  `AddonScope::AcquisitionSearchRead` with stable wire values.
- Protocol DTOs cover search intent, result links, merged links, link taxonomy,
  provider execution status, and provider finality.
- Link debug output redacts raw URLs and extraction passwords.
- `nako-addon-client` exposes typed resource-search helpers over the existing
  generic resource-call transport and scope/schema validation path.
- `nako-server` exposes an admin resource-search diagnostic route that returns
  safe counts/provider summaries without raw result payloads.
- Acquisition intake now has explicit `resource_search_selection` source kinds
  for intake candidates and managed import artifacts.
- `AcquisitionIntakeAppService::record_resource_search_selection` converts a
  selected result/link into a ready intake candidate only after explicit host
  selection.
- Selected-link conversion stores stable hashed source keys and redacted
  diagnostics; it does not run link checks, downloaders, cloud-drive saves,
  media-source creation, or promotion apply.

## Review Result

### Workstream Compliance

- Blocking: none.
- Important: none.
- `ARSP-010` through `ARSP-060` are complete.
- The read-only `acquisition_search_read` scope stays separate from addon
  runtime candidate-write behavior.
- Server routing for selecting a search result was intentionally deferred; the
  lane only introduced the app-service handoff seam.

### Code Quality

- Blocking: none.
- Important: none.
- The new source kind is explicit instead of overloading `addon_proposed`.
- Search-call logic reuses existing resource-call transport and manifest/scope
  validation rather than creating a second addon HTTP path.
- Intake conversion uses existing candidate diagnostics and acceptance flow,
  preserving redaction behavior and managed-import separation.
- DB contract coverage proves the new source kind round-trips for SQLite.

### Missing Gates

- None for this lane.
- Full workspace nextest was not run because this lane touched bounded addon
  protocol/client/server/intake contracts; focused package gates and broad
  `cargo check` covered the changed surfaces.

## Follow-Ons

1. **Official addon migration**
   - In `nako-official-addons`, migrate the resource-search sidecar manifest
     from temporary `automation` to `resource_search`.
   - Grant/request `acquisition_search_read`.
   - Emit the typed request/response schemas added in this lane.
2. **Admin/UI selection flow**
   - Add an operator-facing result view and explicit select action.
   - Wire selection to the existing
     `AcquisitionIntakeAppService::record_resource_search_selection` seam.
3. **Link checking**
   - Define a separate read-only link availability/check contract.
   - Keep link checking separate from search and downloader execution.
4. **Downloader / external acquisition runner**
   - Define a separate execution scope and runtime boundary for magnet, cloud
     drive, and external downloader integrations.
   - Decide how passwords/extraction codes are represented before storing or
     dispatching them.
5. **Cloud-drive save / transfer**
   - Treat cloud-drive save as its own authority and audit trail, not as part of
     resource search.

## Evidence Anchors

- `docs/workstreams/addon-resource-search-protocol/EVIDENCE_AND_GATES.md`
- `crates/nako-addon-protocol/src/lib.rs`
- `crates/nako-addon-client/src/lib.rs`
- `crates/nako-api/src/extension.rs`
- `crates/nako-server/src/app/addons.rs`
- `crates/nako-server/src/app/acquisition_intake.rs`
- `crates/nako-core/src/acquisition_intake.rs`
- `crates/nako-core/src/managed_import.rs`
- `crates/nako-db/src/contract_tests.rs`
