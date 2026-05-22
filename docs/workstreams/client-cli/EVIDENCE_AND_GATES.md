# Client CLI Entrypoint Evidence And Gates

Status: Completed
Last updated: 2026-05-17

## Required Gates

```bash
cargo fmt --all -- --check
cargo check -p nako-client-cli --tests
cargo nextest run -p nako-client-cli --no-fail-fast
cargo tree -p nako-client-cli
cargo check --workspace --tests
cargo nextest run --workspace --no-fail-fast
git diff --check
```

## Boundary Evidence To Record

- `crates/nako-client-cli/Cargo.toml` has `license = "Apache-2.0"`.
- `nako-client-cli` depends on `nako-client`, not on `nako-api`,
  `nako-server`, `nako-core`, `nako-streaming`, or `nako-transcode`.
- Streaming request output redacts bearer token values.
- CLI tests use `nako-client::ClientTransport` mocks instead of a live server.

## Closeout Evidence

- `crates/nako-client-cli/Cargo.toml` uses `license = "Apache-2.0"`.
- `crates/nako-client-cli/src/lib.rs` builds a `nako_client::NakoClient` and
  calls SDK methods or SDK streaming request builders.
- `cargo fmt --all -- --check`: passed.
- `cargo check -p nako-client-cli --tests`: passed.
- `cargo nextest run -p nako-client-cli --no-fail-fast`: 5 tests passed.
- `cargo tree -p nako-client-cli`: passed; manual review shows direct Nako
  dependency is `nako-client`, with `nako-client-protocol` only through the SDK
  and no AGPL server/internal Nako crates.
- `cargo check --workspace --tests`: passed.
- `cargo nextest run --workspace --no-fail-fast`: 279 tests passed.
- `git diff --check`: passed with Git CRLF normalization warnings only.
