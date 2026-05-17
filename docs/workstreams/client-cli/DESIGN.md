# Client CLI Entrypoint

Status: Completed
Last updated: 2026-05-17

## Why This Lane Exists

M35 added the Apache-2.0 Rust SDK foundation, and M36 moved public route
inventory into the Apache-2.0 `taru-client-protocol` crate. The next practical
risk is whether a real external program can consume the SDK without depending
on AGPL server internals or reimplementing HTTP calls by hand.

A small Rust CLI is the lowest-cost concrete client. It can validate base URL
handling, bearer auth, API-version checks, catalog browsing, playback decision
queries, and streaming request construction before Flutter, Web, or package
publishing decisions add platform-specific complexity.

## License Boundary

ADR 0022 remains authoritative.

- `taru-client-cli` is Apache-2.0.
- `taru-client-cli` may depend on `taru-client` and compatible third-party
  runtime crates.
- `taru-client-cli` must not depend on `taru-api`, `taru-server`,
  `taru-core`, `taru-streaming`, `taru-transcode`, or other AGPL server or
  internal Taru crates.
- Public DTOs and route facts must arrive through `taru-client`, not a second
  handwritten client protocol layer.
- `taru-server` keeps its existing server-operator CLI. The new client CLI is
  a separate public-client entrypoint.

## Target State

- A new Apache-2.0 CLI crate exists under `crates/taru-client-cli`.
- The CLI binary talks to Taru through `taru-client`.
- Commands produce JSON output suitable for shell scripts and smoke tests.
- The first command set covers:
  - health/API version preflight;
  - library list;
  - item list and search;
  - source probe;
  - playback decision;
  - playback session get/cancel;
  - streaming request URL/header construction for direct stream, remux, HLS
    playlist, and HLS segment routes.
- Streaming commands print request facts only. They do not execute streaming
  bodies or implement downloads/playback.

## In Scope

- CLI argument parsing for base URL, bearer token, pagination, search, playback
  capability, remux container, and range headers.
- Mock-transport tests proving the CLI goes through `taru-client`.
- Sanitized streaming request output that never prints bearer token values.
- Docs for local use and validation commands.
- Dependency and license evidence through manifest checks and `cargo tree`.

## Out Of Scope

- crates.io publishing, installer scripts, release automation, or shell
  completions.
- Interactive TUI, player integration, HLS playback, download manager, cache,
  retries, or background sync.
- Server-admin/internal commands, scan jobs, metadata maintenance, addon,
  webhook, automation, storage diagnostics, or provider diagnostics.
- Flutter/Dart SDK, Web UI, or mobile client work.

## Architecture Direction

Keep the CLI intentionally thin. It should parse command-line input, construct a
`taru_client::TaruClient`, call SDK methods, and serialize the returned public
DTOs. For streaming routes, it should call SDK request builders and serialize
the resulting method, URL, and safe headers.

Do not introduce a CLI-specific HTTP layer or DTO module. If the CLI needs a
new public API affordance, add it to `taru-client` or
`taru-client-protocol` first and keep the license boundary explicit.

## Closeout Condition

M37 can close when:

- `crates/taru-client-cli` builds and tests as an Apache-2.0 crate;
- the CLI depends on `taru-client` and no AGPL Taru server/internal crates;
- JSON commands and streaming request builders have focused tests;
- docs explain command scope and non-goals;
- all M37 gates pass.
