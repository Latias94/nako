# Public API Contract Hardening

Status: Completed
Last updated: 2026-05-17

## Why This Lane Exists

M29 gave Taru a useful `taru-client-protocol` crate for public browse and
playback DTOs. The next client risk is not another DTO migration; it is
compatibility. Flutter, web, CLI, and SDK consumers need stable API version
identity, error codes, status mappings, and response envelope rules before
client code starts depending on ad hoc server behavior.

## Relevant Authority

- ADRs:
  - `docs/adr/0022-keep-public-protocol-crates-permissive-while-server-crates-remain-agpl.md`
  - `docs/adr/0023-public-api-versioning-and-error-envelope-contract.md`
- Existing docs:
  - `CONTEXT.md`
  - `docs/GOALS.md`
  - `docs/ROADMAP.md`
  - `docs/api/HTTP_API.md`
  - `docs/workstreams/public-client-api/`
- Related crates and modules:
  - `crates/taru-client-protocol`
  - `crates/taru-api`
  - `crates/taru-server/src/http.rs`
  - `crates/taru-server/src/http/error.rs`
  - `crates/taru-server/src/http/query.rs`
  - `crates/taru-server/src/http/system.rs`
  - `crates/taru-server/src/http/tests/*`

## Starting Audit

- `taru-client-protocol` currently owns `HealthResponse`, `ErrorResponse`,
  `PageInfo`, and the M29 browse/playback DTOs.
- `ErrorResponse` currently has `code` and `message` string fields but no
  protocol-owned error-code vocabulary.
- `ApiError` maps `TaruError` into HTTP status, stable-looking code strings,
  and public messages in `crates/taru-server/src/http/error.rs`.
- Pagination rules live in `taru-core::PageRequest`, `taru-server` query
  parsing, and `taru-client-protocol::PageInfo`.
- `/health` returns `version: API_VERSION`, where `API_VERSION` re-exports
  `CLIENT_PROTOCOL_VERSION`.
- Route tests cover several error codes and pagination cases, but the public
  route contract is not explicitly separated from server-admin/internal
  routes.

## Problem

- Clients should not parse free-form error messages, but protocol-owned error
  code types do not exist yet.
- The public client API route subset is not documented as a compatibility
  surface distinct from admin/internal diagnostics.
- API version identity is visible through `/health`, but the response-header
  strategy and future path-versioning policy need to be made durable.
- Pagination is mostly stable but not tied to the public API contract in a
  single place.
- Current tests prove behavior locally, but M30 needs tests that intentionally
  protect public route JSON and error codes.

## Target State

- `taru-client-protocol` owns a stable v1 error-code vocabulary and public
  error envelope.
- `taru-api` re-exports protocol error types and remains the server adapter.
- `taru-server` maps `TaruError` into stable public error codes, safe messages,
  and HTTP status codes through one auditable boundary.
- Public client routes have focused tests for success envelope shape,
  pagination, API version identity, and error status/code behavior.
- Server-admin/internal routes are explicitly documented as outside the first
  public compatibility promise, even if they use the same baseline error
  envelope.
- `cargo tree -p taru-client-protocol` continues to prove the protocol crate
  has no internal server dependencies.

## In Scope

- Add protocol-owned public error code types or constants.
- Keep the v1 error envelope compatible with the current `code/message` JSON
  shape.
- Audit and document API version identity, public route subset, pagination
  rules, and status/code mappings.
- Add or update route-level tests for public catalog, library, playback, and
  system routes.
- Update HTTP API docs to reflect the public v1 contract.

## Out Of Scope

- Flutter, web, or CLI client implementation.
- OpenAPI or SDK generation.
- Authentication or authorization redesign.
- Route path rewrite or immediate `/api/v1` migration.
- Moving all server-admin/internal DTOs into `taru-client-protocol`.
- Full migration of diagnostics, jobs, provider debug, webhook, automation,
  addon administration, ingestion failures, or metadata maintenance DTOs.

## Starting Assumptions

| Assumption | Confidence | Evidence | Consequence if wrong |
| --- | --- | --- | --- |
| Public clients should branch on error codes, not messages. | High | Current `ErrorResponse` already has `code`. | Client behavior would be brittle. |
| v1 can start with `/health.version`, `x-taru-api-version`, and protocol constants before path versioning. | Medium | Only one API version exists today. | Need a follow-on ADR for multi-version route/header negotiation. |
| Existing `code/message` JSON must remain compatible. | High | Current tests and docs already use it. | Breaking clients before they exist adds needless churn. |
| Admin/internal surfaces should not block public route hardening. | High | M29 explicitly left admin DTOs out. | M30 would become too broad to close cleanly. |

## Architecture Direction

Keep the public contract narrow and explicit. The first M30 implementation
should preserve the existing JSON shape while replacing untyped string
knowledge with protocol-owned constants or enums. The server may still derive
codes from `TaruError`, but external clients should see only the stable public
vocabulary.

## Closeout Condition

This lane can close when:

- the public v1 version/error/pagination rules are documented;
- the protocol crate owns the public error-code vocabulary without internal
  dependencies;
- `taru-server` error mapping is covered by route or adapter tests for public
  codes;
- public catalog/library/playback/system route tests protect envelope behavior;
- `docs/api/HTTP_API.md`, `docs/GOALS.md`, `docs/ROADMAP.md`, and workstream
  docs agree;
- full validation gates pass;
- and follow-ons are explicitly recorded.

## Closeout Summary

M30 closed after adding protocol-owned public error codes, preserving the v1
`code/message` error envelope, advertising `v1` through `/health.version` and
the `x-taru-api-version` response header, and extending route tests for public
pagination, version identity, and stable error-code behavior. Path versioning,
OpenAPI, SDK generation, auth redesign, and broader admin/internal DTO
migration remain follow-ons.
