# Addon Resource Link Check Contract

Status: Active
Last updated: 2026-05-28

## Why This Lane Exists

Resource search is read-only discovery. The next adjacent capability is checking
whether a selected external resource link is reachable, unsupported,
rate-limited, or needs a password/code. That must not reuse
`acquisition_search_read`, because a link check has its own timeout, retry,
cache, redaction, and provider-touch behavior.

## Relevant Authority

- ADR:
  - `docs/adr/0050-acquisition-resource-action-boundaries.md`
- Related workstreams:
  - `docs/workstreams/addon-resource-search-protocol/`
  - `docs/workstreams/addon-resource-search-product-flow/`
  - `../nako-official-addons/docs/workstreams/official-resource-search-first-class-protocol/`

## Problem

Nako can search external resources and record selected links into acquisition
intake, but there is no first-class addon contract for asking whether a
host-owned selected link is usable. Without a separate contract, future
implementations would be tempted to overload search or generic automation.

## Target State

- `nako-addon-protocol` declares a first-class `resource_link_check` resource.
- A dedicated read-only scope authorizes link-check calls.
- Request and response DTOs are typed and versioned.
- Responses expose safe facts only, not raw URLs or passwords.
- `nako-addon-client` has a typed helper that validates manifest resource,
  granted scope, request schema, response schema, and response payload shape.

## In Scope

- Protocol vocabulary, schemas, DTOs, and tests.
- Addon client helper and tests.
- Documentation and evidence for the contract slice.

## Out Of Scope

- Admin UI.
- Product API route.
- Actual link-check provider implementation.
- Downloader execution.
- Cloud-drive save/transfer.
- Durable password/code persistence.

## Starting Assumptions

| Assumption | Confidence | Evidence | Consequence if wrong |
| --- | --- | --- | --- |
| Link checking should be a resource, not a task, for the first contract slice. | Medium | It is a bounded read-only request/response probe similar to search. | If checks need long-running scheduling, add a task wrapper later without changing search. |
| The host may send raw selected-link data to an addon, but browser clients must not. | High | Resource-search product flow already keeps raw links in host-owned sessions. | Product route must consume opaque selection IDs, not browser-submitted URLs. |
| Response facts should be safe and redaction-first. | High | ADR 0050 and search flow prohibit raw URL/password leakage. | Additive fields must be reviewed before public API exposure. |

## Architecture Direction

The first slice mirrors the resource-search protocol/client pattern:

```text
AddonResource::ResourceLinkCheck
  + AddonScope::AcquisitionLinkCheckRead
  + AddonResourceLinkCheckRequest/Response
  + call_addon_resource_link_check helper
```

The request may carry an `AddonResourceLink` because host-to-addon calls are
trusted addon protocol traffic. Browser/product APIs must remain separate and
must pass only opaque selection references.

## Closeout Condition

This lane can close when:

- protocol and client helpers compile and pass targeted tests,
- docs record non-goals,
- evidence gates are fresh,
- and follow-on server/product integration is explicitly deferred.
