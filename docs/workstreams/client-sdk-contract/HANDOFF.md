# Client SDK Contract Inventory And Streaming Builders Handoff

Status: Completed
Last updated: 2026-05-17

## Current State

M36 is closed. Shared public route inventory lives in `taru-client-protocol`,
and Rust SDK streaming support is request-builder only.

## Active Task

- None.

## Decisions Since Last Update

- Public route inventory belongs in `taru-client-protocol` because it is a
  neutral public contract artifact and must remain reusable by clients.
- `taru-api` remains the OpenAPI renderer and TypeScript SDK generator.
- Rust streaming support starts as request builders, not response body
  execution.
- The shared protocol inventory has 24 public routes: 20 JSON-method routes
  and 4 streaming-builder routes.
- Rust SDK streaming support is request-builder only and covers direct stream,
  HEAD preflight, remux stream, HLS playlist, and HLS segment requests.

## Blockers

- None.

## Closeout Validation

- `cargo fmt --all -- --check`: passed.
- `cargo check -p taru-client-protocol --tests`: passed.
- `cargo check -p taru-api --tests`: passed.
- `cargo check -p taru-client --tests`: passed.
- `cargo nextest run -p taru-client-protocol --no-fail-fast`: 7 tests passed.
- `cargo nextest run -p taru-api --no-fail-fast`: 11 tests passed.
- `cargo nextest run -p taru-client --no-fail-fast`: 8 tests passed.
- `cargo nextest run -p taru-server http::tests::playback --no-fail-fast`: 16
  tests passed.
- `cargo check --workspace --tests`: passed.
- `cargo nextest run --workspace --no-fail-fast`: 274 tests passed.
- `cargo tree -p taru-client-protocol`: passed.
- `cargo tree -p taru-client`: passed.
- `npm run check --prefix sdk/typescript`: passed.
- `git diff --check`: passed with Git CRLF normalization warnings only.

## Next Recommended Action

- Choose M37 around Rust CLI, Flutter/Dart SDK, or full Rust streaming body
  abstraction.
