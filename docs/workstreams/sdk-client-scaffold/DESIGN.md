# SDK Generation And Client Integration Scaffold

Status: Completed
Last updated: 2026-05-17

## Why This Lane Exists

M32 created a machine-readable Public Client API contract. The next risk is
that each future client still hand-rolls auth headers, error parsing,
pagination parameters, and route paths. M33 creates the first repeatable SDK
generation path and smoke checks before committing to Flutter, web UI, or
published packages.

## Relevant Authority

- ADRs:
  - `docs/adr/0022-keep-public-protocol-crates-permissive-while-server-crates-remain-agpl.md`
  - `docs/adr/0023-public-api-versioning-and-error-envelope-contract.md`
  - `docs/adr/0024-inbound-token-authentication-boundary.md`
  - `docs/adr/0025-openapi-public-client-sdk-contract.md`
- Existing workstreams:
  - `docs/workstreams/openapi-client-contract/`
  - `docs/workstreams/public-client-api/`
  - `docs/workstreams/public-api-contract/`
  - `docs/workstreams/access-boundary-auth/`
- Code boundaries:
  - `crates/nako-api/src/openapi.rs`
  - `crates/nako-api/examples/emit-openapi.rs`
  - `crates/nako-api/src/sdk.rs`
  - `docs/api/HTTP_API.md`

## Starting Audit

- `nako-api` can emit OpenAPI v1 JSON.
- There is no SDK generation command yet.
- There is no client wrapper that standardizes bearer auth, API version
  checks, error envelope parsing, or core route calls.
- The repo has no Node, Dart, or Flutter package layout yet.

## Target State

- M33 has a durable workstream with explicit SDK boundaries.
- `nako-api` can emit a TypeScript SDK scaffold from the M32 OpenAPI document.
- The TypeScript scaffold has:
  - dependency-free `fetch` integration for web/CLI runtimes;
  - bearer token header injection;
  - `x-nako-api-version` response inspection;
  - `ErrorResponse` parsing into a typed API error;
  - page query helpers;
  - core library, catalog, playback, and playback-session calls.
- Static tests ensure the SDK scaffold stays aligned with the OpenAPI public
  route inventory and excludes admin/internal route groups.
- Docs record generation and validation commands.

## In Scope

- Generate a lightweight TypeScript/Web/CLI SDK scaffold from `nako-api`.
- Add tests for SDK route coverage, auth/error/version behavior text, and
  admin/internal leakage rejection.
- Keep generated output as a command result rather than a published package.
- Update HTTP API docs, goal map, roadmap, and workstream index.

## Out Of Scope

- Flutter, web, or CLI UI implementation.
- Publishing npm or pub packages.
- Full browser/e2e tests.
- User accounts, sessions, OAuth/OIDC, LDAP, Passkey, or RBAC.
- Server-admin/internal SDK coverage.
- Expanding server public API behavior beyond small contract hygiene.

## Architecture Direction

Keep SDK generation in `nako-api` so it depends on the OpenAPI aggregation
boundary, not on server route handlers. Avoid Node/Java generator dependencies
in the first slice. The TypeScript output is a scaffold for integration
validation and future package extraction, not a committed external SDK release.

## Closeout Condition

This lane can close when:

- SDK generation docs and task ledger are complete;
- `cargo run -p nako-api --example emit-typescript-sdk` emits a usable
  TypeScript scaffold;
- tests prove route inventory coverage, auth/error/version behavior, and no
  admin/internal leakage;
- HTTP API docs record generation/validation commands;
- full validation gates pass;
- and SDK publishing, Dart/Flutter, OpenAPI route serving, and real client UI
  are recorded as follow-ons.
