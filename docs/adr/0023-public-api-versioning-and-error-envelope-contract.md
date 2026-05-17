# 0023: Stabilize Public API Versions and Error Envelopes

## Status

Accepted.

## Context

M29 moved the first useful public client DTOs into
`taru-client-protocol`. Future Flutter, web, CLI, and SDK consumers now have a
permissive crate boundary for browse and playback response shapes, but the
HTTP contract still needs a stable compatibility policy.

The current server already has a small error envelope with `code` and
`message`, offset pagination with `limit`, `offset`, and `returned`, and a
`/health` response that reports protocol version `v1`. Those shapes are useful
but under-specified:

- there is no durable rule for how public API versions are advertised;
- error codes are strings without a protocol-owned vocabulary;
- public client routes and server-admin routes are not explicitly separated
  for error and envelope compatibility;
- internal storage/provider/database details are hidden inconsistently by
  convention rather than by documented contract.

## Decision

Taru will treat the public client HTTP contract as a versioned `v1` surface
backed by `taru-client-protocol`. The server advertises the active public
contract version through `GET /health` and the `x-taru-api-version` response
header. Route path versioning is deferred until Taru needs multiple
concurrently supported API versions.

Public client responses must use protocol-owned wire types where the response
shape is intended for Flutter, web, CLI, or SDK reuse. IDs remain strings at
the public boundary. Pagination remains offset based for v1:

- request fields: `limit` and `offset`;
- response fields: `limit`, `offset`, and `returned`;
- `returned` is the current page item count, not a total count.

Error responses must use a stable protocol-owned envelope. The v1 minimum
shape is:

```json
{
  "code": "not_found",
  "message": "not found: item 018f..."
}
```

The server may add optional forward-compatible fields later, but existing v1
fields must keep their meaning. Public error codes are stable strings and must
not expose internal Rust enum names, database errors, raw provider messages, or
plaintext storage credentials.

Public client routes and server-admin/internal routes share the same baseline
error envelope, but only the public client route subset is part of the stable
client compatibility promise. Admin diagnostics, job internals, provider
runtime details, webhook, automation, addon administration, ingestion failure,
and metadata maintenance DTOs remain server/API-owned until a later goal moves
them into a public protocol contract.

## Consequences

- Client applications can handle errors by code instead of parsing messages.
- Taru can evolve internal `TaruError`, storage providers, and playback
  services without changing public client error codes.
- Route tests must cover both HTTP status and stable error code for public
  routes.
- `taru-client-protocol` must remain dependency-light and cannot import
  `taru-core`, `taru-streaming`, `taru-transcode`, or `taru-server`.
- Adding OpenAPI output or generated SDKs can happen as follow-ons without
  changing the baseline v1 rule.

## Alternatives Considered

- Put the API contract in `taru-api`: rejected because that keeps clients tied
  to the AGPL server adapter crate.
- Use raw `TaruError` variants as public codes: rejected because internal
  domain/runtime changes would become client breaking changes.
- Add a total count to all paginated responses now: rejected because it adds
  unnecessary database cost and does not solve the current compatibility
  problem.
- Version every route path immediately, such as `/api/v1/...`: deferred. Taru
  can advertise `v1` first and add path or header negotiation when multiple
  concurrently supported versions exist.

## Related Workstreams

- `docs/workstreams/public-client-api/`
- `docs/workstreams/public-api-contract/`
