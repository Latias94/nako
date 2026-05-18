# Public Client Source Locator Redaction

Status: Proposed
Last updated: 2026-05-18

## Why This Lane Exists

Public Client DTOs include source locators that were useful during early local
debugging. That is now a contract risk. A **Source Locator** is a
library-scoped address Taru uses to find a **Media Source**; it is not a stable
client identity, and it may reveal local paths, remote storage layout, bucket
names, or naming conventions.

This lane is routed from ARF-005 in the 2026-05-18 architecture review.

## Relevant Authority

- `CONTEXT.md`
- `docs/adr/0022-keep-public-protocol-crates-permissive-while-server-crates-remain-agpl.md`
- `docs/adr/0023-public-api-versioning-and-error-envelope-contract.md`
- `docs/adr/0025-openapi-public-client-sdk-contract.md`
- `docs/adr/0027-admin-api-boundary-for-web-console.md`
- `docs/workstreams/public-client-api/`
- `docs/workstreams/public-api-contract/`
- `docs/workstreams/openapi-client-contract/`
- `docs/workstreams/api-module-split/`

## Problem

Current public protocol and OpenAPI shapes include `MediaSourceDto.locator` and
`ClientTranscodePlan.input_locator`. The server also maps internal
`MediaSource.locator` directly into public DTOs. That mixes internal storage
addressing with **Public Client API** contracts.

The public contract should let clients identify and play sources through stable
IDs and server-owned playback routes. It should not require or expose the
storage locator that Taru uses internally.

## Target State

- Public Client source DTOs do not expose raw Source Locators.
- Playback/transcode public DTOs do not expose raw input locators.
- Admin diagnostics may expose redacted locator summaries when useful, but the
  Public Client API does not.
- OpenAPI and generated SDK artifacts reflect the public redaction contract.
- Tests prove public JSON omits locator fields while internal/server workflows
  still retain locators for storage and playback execution.

## In Scope

- Audit public DTO fields for raw locator leakage.
- Define a safe replacement shape, such as source IDs, file names, display
  labels, backend kind, or redacted locator summaries where appropriate.
- Update `taru-client-protocol`, `taru-api`, OpenAPI generation, SDK generation,
  and HTTP route tests if public wire shape changes.
- Keep internal `MediaSource.locator` unchanged.
- Update HTTP API docs and workstream evidence.

## Out Of Scope

- Media Library source-of-truth reconciliation.
- RBAC or Library Access enforcement.
- Storage backend redesign.
- Playback source selection algorithm changes.
- Admin diagnostics hardening beyond locator redaction decisions.
- Full API path-version migration.

## Starting Assumptions

| Assumption | Confidence | Evidence | Consequence if wrong |
| --- | --- | --- | --- |
| Public clients can use `MediaSourceId` and playback routes instead of raw locators. | High | Existing playback APIs operate through source/session IDs. | Need a compatibility field or migration note if a generated client requires the locator. |
| Admin APIs may need redacted locator diagnostics. | Medium | Admin diagnostics already expose storage/job details. | Split an Admin API diagnostics follow-up if redaction policy differs by surface. |
| Removing public locator fields is acceptable before stable external clients exist. | Medium | Taru is still pre-compatibility for this specific shape. | If compatibility matters, deprecate fields first and add tests for transitional behavior. |
| OpenAPI/SDK tests must change with DTO shape. | High | OpenAPI and TypeScript/Rust SDK lanes generate from protocol DTO inventory. | Public docs and generated clients drift from server behavior. |

## Architecture Direction

Keep locators internal. The server can translate internal **Source Locator**
values into safe public facts through `taru-api` mapping functions. The
protocol crate should own the public shape, and OpenAPI/SDK artifacts should be
generated from that shape.

The first executable task should be an audit and contract decision, not an
immediate field removal. The audit should classify each locator exposure as
Public Client, Admin API, internal test fixture, or server-only playback
execution.

## Closeout Condition

This lane can close when:

- every public locator exposure is removed, redacted, or explicitly justified;
- route/OpenAPI/SDK tests prove the public contract;
- internal storage/playback workflows still receive full locators;
- docs describe the redaction policy;
- and any compatibility or Admin API follow-ons are split.
