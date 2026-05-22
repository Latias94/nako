# 0025: Generate Public Client OpenAPI From Protocol-Owned Wire Types

Status: accepted

## Context

Nako now has a permissive public client protocol crate, stable HTTP error
envelopes, and an inbound bearer-token boundary. Future Flutter, web, CLI, and
SDK work needs a machine-readable API contract instead of reverse-engineering
server routes, handwritten docs, or AGPL server internals.

The first OpenAPI contract must not make `nako-core`, `nako-server`,
provider diagnostics, job internals, raw metadata cache, addon administration,
webhook administration, automation administration, storage diagnostics, local
filesystem paths, or secret references part of the Public Client API.

## Decision

Nako will generate the first Public Client API OpenAPI v1 contract from the
AGPL `nako-api` adapter layer while keeping the wire DTO source of truth in
`nako-client-protocol`.

- `nako-client-protocol` owns permissive public wire DTOs, public error codes,
  API version constants, and client-facing enum vocabularies.
- `nako-api` owns OpenAPI aggregation, server-domain-to-protocol mapping, and
  any schema helper code that would otherwise add non-essential dependencies to
  the protocol crate.
- `nako-server` owns HTTP route wiring and behavior tests, but OpenAPI must not
  reference server handler types or internal domain records.
- The first OpenAPI v1 artifact covers only the public client route set:
  health, libraries, catalog browse/search, source probe, playback decision,
  direct/remux/HLS playback surfaces, playback session inspection, playback
  cancellation, and HLS segment fetch.
- Server-admin/internal routes remain documented separately until a distinct
  admin API contract is accepted.

Bearer auth, the `x-nako-api-version` header, `ErrorResponse`, pagination, and
common public error responses are part of the generated contract.

## Consequences

- Public client routes cannot keep returning DTOs that expose local paths or
  internal record shapes merely because those shapes were convenient for the
  server.
- The OpenAPI generator gets tests that reject internal crate names, admin-only
  route groups, secret references, raw provider cache wording, and local path
  fields.
- `nako-client-protocol` stays dependency-light by default. If future schema
  derives are needed, they must be optional or live behind the `nako-api`
  aggregation boundary.
- OpenAPI drift becomes a testable compatibility risk instead of a manual docs
  hygiene problem.

## Alternatives Considered

- Generate OpenAPI directly from `nako-server` route handlers. Rejected because
  it couples public client contracts to Axum/server internals and makes
  admin/internal leakage likely.
- Add unconditional OpenAPI derive dependencies to `nako-client-protocol`.
  Rejected for the first slice because the protocol crate should remain small
  and permissive-client friendly by default.
- Continue with prose-only HTTP docs. Rejected because SDK/client generation
  needs a machine-readable contract.

## Related Workstreams

- `docs/workstreams/openapi-client-contract/`
- `docs/workstreams/public-client-api/`
- `docs/workstreams/public-api-contract/`
- `docs/workstreams/access-boundary-auth/`
