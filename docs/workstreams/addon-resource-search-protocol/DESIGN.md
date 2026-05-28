# Addon Resource Search Protocol

Status: Closed
Last updated: 2026-05-28

## Why This Lane Exists

The official resource-search sidecar currently declares an `automation`
resource because the Addon Protocol has no first-class read-only search
resource for acquisition candidates. That temporary shape keeps the sidecar
usable, but it hides the real permission boundary and makes downloader or
acquisition side effects too easy to blur into search.

## Relevant Authority

- ADRs:
  - `docs/adr/0015-capability-scoped-http-addons-and-automation-providers.md`
  - `docs/adr/0020-jellyfin-like-sidecar-addons-with-scoped-nako-api-access.md`
  - `docs/adr/0033-version-addon-protocol-independently-from-addon-and-crate-releases.md`
  - `docs/adr/0034-package-addon-capabilities-into-sidecar-suites.md`
- Existing docs:
  - `CONTEXT.md`
  - `F:/SourceCodes/Rust/nako-official-addons/docs/workstreams/official-resource-search-architecture-hardening/PROTOCOL_PROPOSAL.md`
- Related workstreams:
  - `docs/workstreams/addon-architecture-deepening`
  - `docs/workstreams/addon-runtime-and-distribution`
  - `docs/workstreams/addon-task-runtime-contract`
  - `docs/workstreams/official-addon-e2e-alpha2`

## Problem

Resource discovery for external links is a read-only addon capability, not an
automation job and not a download. Without a dedicated protocol resource and
scope, the host cannot express:

- that a sidecar may search external sources but may not write candidates;
- that returned links are acquisition candidates, not trusted playback URLs;
- that provider execution diagnostics must remain redaction-safe;
- that link checking and downloader execution are separate contracts.

## Target State

When this lane closes:

- `nako-addon-protocol` has a first-class `AddonResource::ResourceSearch` wire
  value `resource_search`.
- `nako-addon-protocol` has an `AddonScope::AcquisitionSearchRead` wire value
  `acquisition_search_read`.
- Resource-search request and response DTOs make search intent, link types,
  provider execution status, finality, and result links explicit.
- Manifest validation and addon resource response validation cover the new
  resource and scope.
- `nako-addon-client` can perform a typed resource-search call while preserving
  existing timeout, retry, protocol-version, and redaction behavior.
- Host-side server work has a first app-service seam for converting selected
  search results into `resource_search_selection` acquisition intake
  candidates.

## In Scope

- Addon Protocol enum, scope, DTO, and validation tests.
- Typed client helper for resource-search calls if it stays within the existing
  generic resource-call machinery.
- Server-side planning for bounded host calls and acquisition handoff.
- Host-owned selected-result conversion into intake candidates without HTTP
  routing, downloader execution, or cloud-drive save behavior.
- Documentation of the permission split between search, link checking,
  candidate writing, and downloader execution.

## Out Of Scope

- Downloader execution, cloud-drive save actions, or link availability checks.
- A write scope for acquisition candidate submission.
- Migrating `nako-resource-search` away from the temporary `automation`
  declaration; that is a follow-on in `nako-official-addons`.
- Site-specific scraper implementations.
- Addon Manager install/update/process supervision changes.

## Starting Assumptions

| Assumption | Confidence | Evidence | Consequence if wrong |
| --- | --- | --- | --- |
| Resource search is read-only and should not imply acquisition candidate writes. | High | ADR 0015/0020 and the official addon proposal separate external fetches from Nako-managed artifacts. | Add a separate write scope and do not broaden `acquisition_search_read`. |
| Adding an enum value and optional DTOs is compatible within the current alpha supported protocol version. | Medium | ADR 0033 allows compatible additions but breaking semantics need explicit versioning. | Bump or split Addon Protocol Version before accepting manifests. |
| Generic addon resource calls can carry typed DTOs without a new transport path. | High | `nako-addon-client` already validates resource declarations, scope grants, envelopes, retry, and protocol version. | Introduce a thin typed helper over the existing resource call rather than a new transport. |
| Search results should become acquisition intake candidates only after host/user selection. | High | `AcquisitionIntakeCandidate` already models host-owned intake state. | Keep conversion as a separate server task and avoid direct side effects in the search response. |

## Architecture Direction

`nako-addon-protocol` owns the public wire vocabulary: resource kind, scope,
request DTOs, response DTOs, link taxonomy, provider execution diagnostics,
and validation tests.

`nako-addon-client` owns typed call ergonomics, but should keep using the
existing `call_addon_resource_with_outcome` machinery for retries, headers,
authentication, protocol validation, and safe error mapping.

`nako-server` owns host policy: which addon may be called, what scopes are
granted, result limits, redaction, and how a selected result is converted into
an `AcquisitionIntakeCandidate`. Selected links use
`resource_search_selection` source kinds so read-only search output is not
confused with addon runtime candidate writes. Search does not call downloaders
directly.

`nako-official-addons` remains the adapter implementation boundary. It can map
host DTOs to its internal query model after the host protocol lands.

## Closeout Condition

This lane is closed. It satisfied:

- the protocol and typed client slices are implemented and tested,
- host-side handoff behavior is either implemented or split to a named follow-on,
- docs record the delivered behavior and deferred downloader/link-check scopes,
- focused gates pass,
- and the official addon migration path is explicit.
