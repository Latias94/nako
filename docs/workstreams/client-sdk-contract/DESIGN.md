# Client SDK Contract Inventory And Streaming Builders

Status: Completed
Last updated: 2026-05-17

## Why This Lane Exists

M32-M35 established the public OpenAPI contract, TypeScript SDK package, and
Rust SDK foundation. The remaining contract risk is duplication: `nako-api`
owns one public route inventory while `nako-client` owns a second SDK inventory.

That duplication matters because the route inventory is part of the public
client contract. It should be reusable by TypeScript generation, Rust SDK
tests, OpenAPI checks, and future SDKs without making clients depend on the
AGPL server adapter crate.

## License Boundary

ADR 0022 is authoritative for this lane.

- `nako-client-protocol` remains `Apache-2.0` and dependency-light.
- `nako-client` remains `Apache-2.0` and may depend on
  `nako-client-protocol`.
- `nako-api` remains `AGPL-3.0-or-later` and may consume protocol types as a
  server/API adapter.
- Client-reusable inventory or helper types must not live only in `nako-api`,
  because that would make AGPL server code the reuse source for SDKs.
- `nako-client-protocol` must not depend on `http`, `reqwest`, `nako-api`, or
  server/internal crates.

## Target State

- Public client route inventory lives in `nako-client-protocol`.
- `nako-api` OpenAPI and TypeScript SDK generation consume that shared
  inventory instead of a local path list.
- `nako-client` consumes the same inventory and differentiates JSON methods
  from streaming/raw byte request builders.
- Rust SDK request builders can construct:
  - direct stream GET;
  - direct stream HEAD preflight;
  - remux stream GET;
  - HLS playlist GET;
  - HLS segment GET.
- Builders reuse SDK URL normalization, bearer auth, path encoding, range
  headers, playback capability query serialization, and remux output container
  query serialization.

## In Scope

- Add shared public route inventory types/constants to `nako-client-protocol`.
- Keep the protocol crate dependency-light and Apache-2.0.
- Refactor `nako-api` OpenAPI checks and TypeScript SDK generation to use the
  shared inventory.
- Refactor `nako-client` inventory tests to use the same source.
- Add streaming request builders without performing HTTP body streaming.
- Update API docs, goal map, roadmap, workstream index, and evidence.

## Out Of Scope

- crates.io or npm publishing.
- Full streaming response abstraction, download manager, HLS player, or
  playback UI.
- Flutter/Dart SDK, Rust CLI product commands, Web/mobile UI.
- Server public API behavior expansion.
- Any dependency from permissive protocol/SDK crates to AGPL server/internal
  crates.

## Architecture Direction

Use `nako-client-protocol` for neutral contract facts only: route path,
supported method, route kind, and whether the Rust SDK should expose a JSON
method or a request builder. Keep HTTP implementation details in SDK crates.

`nako-api` should remain free to render OpenAPI operations and schema details,
but route membership must come from the shared protocol inventory.

`nako-client` should expose concrete `ClientRequest` builders first. That gives
Rust callers stable URLs/headers/methods for streaming routes without deciding
on byte-stream ownership, retries, range download policy, or HLS playback.

## Closeout Condition

M36 can close when:

- `nako-client-protocol` owns the public inventory and stays dependency-light;
- `nako-api`, TypeScript SDK generation, and `nako-client` consume the shared
  inventory;
- Rust streaming builders are covered by mock/unit tests;
- leakage tests still reject admin/internal/secret/local-path surfaces;
- docs record the license boundary and the non-goals;
- all M36 gates pass.
