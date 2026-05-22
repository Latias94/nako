# Rust Client SDK Foundation Handoff

Status: Completed
Last updated: 2026-05-17

## Current State

M35 is closed. `crates/nako-client` is the first Rust SDK foundation crate for
public Nako client consumers.

## Active Task

- None.

## Decisions Since Last Update

- Rust SDK crate name is `nako-client`.
- License is explicitly `Apache-2.0`, not inherited from the AGPL workspace
  package default.
- DTOs come from `nako-client-protocol`.
- `nako-client` must not depend on `nako-api`, because `nako-api` is an AGPL
  adapter crate that depends on server/domain internals.
- Full streaming/raw body APIs are deferred from the first foundation slice.
- OpenAPI/public route inventory duplication remains local to M35; a follow-on
  can move the public inventory into `nako-client-protocol` if drift cost grows.

## Blockers

- None.

## Closeout Validation

- `cargo fmt --all -- --check`: passed.
- `cargo check -p nako-client --tests`: passed.
- `cargo nextest run -p nako-client --no-fail-fast`: 7 tests passed.
- `cargo tree -p nako-client`: passed; no `nako-core`, `nako-api`,
  `nako-server`, `nako-streaming`, or `nako-transcode` dependency.
- `cargo tree -p nako-client-protocol`: passed; protocol dependency tree is
  still light.
- `npm run check --prefix sdk/typescript`: passed.
- `cargo check --workspace --tests`: passed.
- `cargo nextest run --workspace --no-fail-fast`: 271 tests passed.
- `git diff --check`: passed with Git CRLF normalization warnings only.

## Next Recommended Action

- Open M36 around client SDK inventory extraction and streaming request
  builders, or defer SDK work and move to concrete client/CLI planning.
