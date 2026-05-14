# 0003: Prefer HTTP Addons Before In-Process Plugins

## Status

Proposed

## Context

Taru should support extension points, but Rust does not provide a stable native
plugin ABI for arbitrary dynamic libraries. In-process plugins also raise
sandboxing, crash isolation, versioning, and trust problems.

Stremio demonstrates a useful model where addons are external HTTP services
declaring capabilities through a manifest. That model is language-agnostic and
works well with JavaScript/TypeScript SDKs, serverless deployments, and self-
hosted sidecars.

## Decision

Define a Taru HTTP addon protocol before native plugins. Addons should declare
manifest metadata, supported resources, authentication needs, and response
schemas. Taru can later provide SDKs for JavaScript/TypeScript and Rust, but
the server runtime should speak protocol, not execute addon code directly.

Initial resources may include:

- metadata
- image
- subtitle
- stream
- catalog
- recommendation
- automation
- webhook

## Consequences

- Addons can be built in any language.
- Taru can enforce timeout, retry, authentication, and trust boundaries.
- Addon execution failures do not crash the server.
- Local-only users may need a sidecar process for custom addons.
- A separate compatibility layer can expose Taru content to Stremio clients.

## Alternatives Considered

- Native dynamic library plugins: high power, high risk, unstable ABI concerns.
- WASI plugins: promising, but introduces runtime and capability design work.
- JavaScript runtime embedded in Taru: convenient for JS authors, but adds
  sandboxing and operational complexity.

## Related Workstreams

- `docs/workstreams/server-foundation/`
