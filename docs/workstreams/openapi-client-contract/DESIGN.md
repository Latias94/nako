# OpenAPI And Public Client SDK Contract

Status: Completed
Last updated: 2026-05-17

## Why This Lane Exists

Nako has a public protocol crate, stable API version/error semantics, and
inbound bearer auth. The next client-readiness risk is that future Flutter,
web, CLI, and SDK work would still need to infer HTTP shapes from prose docs,
server tests, or internal DTOs.

## Relevant Authority

- ADRs:
  - `docs/adr/0022-keep-public-protocol-crates-permissive-while-server-crates-remain-agpl.md`
  - `docs/adr/0023-public-api-versioning-and-error-envelope-contract.md`
  - `docs/adr/0024-inbound-token-authentication-boundary.md`
  - `docs/adr/0025-openapi-public-client-sdk-contract.md`
- Existing workstreams:
  - `docs/workstreams/public-client-api/`
  - `docs/workstreams/public-api-contract/`
  - `docs/workstreams/access-boundary-auth/`
- Code boundaries:
  - `crates/nako-client-protocol`
  - `crates/nako-api`
  - `crates/nako-server/src/http`
  - `docs/api/HTTP_API.md`

## Starting Audit

- Public browse/playback DTOs mostly live in `nako-client-protocol`.
- `nako-api` adapts server/domain records into public protocol DTOs.
- Playback session HTTP routes still return a `nako-api` DTO that includes
  `output_path`, which is a server-local staging path and should not be part of
  a generated public client schema.
- There is no OpenAPI generator, artifact, or schema checker yet.
- `docs/api/HTTP_API.md` is the current human-readable contract.

## Target State

- M32 has an accepted ADR and a durable workstream.
- Public playback session DTOs are owned by `nako-client-protocol` and do not
  expose local output paths.
- `nako-api` owns a Public Client API OpenAPI v1 artifact/generator over
  protocol-owned DTOs and server adapter knowledge.
- The OpenAPI contract includes:
  - `x-nako-api-version`;
  - bearer auth with `GET /health` unauthenticated;
  - `ErrorResponse` and stable public error codes;
  - pagination parameters and `PageInfo`;
  - public client library/catalog/search/probe/playback/session routes.
- Tests reject internal crate names, server/admin-only route groups, secret
  references, raw provider cache, job internals, and local path fields in the
  public spec.
- Docs explain how to generate/check the contract and what is intentionally
  excluded.

## In Scope

- Create ADR/workstream docs for the OpenAPI/client SDK contract.
- Move any remaining public client response shape needed by OpenAPI out of
  server/internal record types.
- Implement a first OpenAPI v1 artifact or generator in `nako-api`.
- Add tests/checkers for route coverage, auth/error/header semantics, and
  internal/admin leakage.
- Update HTTP API docs, goal map, roadmap, and workstream index.

## Out Of Scope

- Flutter, web, CLI, or full SDK implementation.
- Publishing generated SDK packages.
- User accounts, sessions, OAuth/OIDC, LDAP, Passkey, or RBAC.
- New browse facets, rating/user-state features, or source variant UI.
- Full server-admin/internal OpenAPI coverage.

## Architecture Direction

Use `nako-api` as the schema aggregation boundary. Do not make Axum handlers or
`nako-core` records the public schema source. Keep `nako-client-protocol`
dependency-light by default; if schema derive support becomes worthwhile, add
it as an optional protocol feature or keep it entirely in `nako-api`.

## Closeout Condition

This lane can close when:

- ADR 0025 and workstream docs define the contract boundary;
- all OpenAPI-covered response DTOs are protocol-owned or explicitly adapted;
- a Public Client API OpenAPI v1 artifact/generator exists;
- checker tests prove public route coverage and reject internal/admin leakage;
- HTTP API docs link the OpenAPI contract and generation/checking workflow;
- full validation gates pass;
- and follow-ons are explicitly recorded.
