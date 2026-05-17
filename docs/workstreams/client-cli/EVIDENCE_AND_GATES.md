# Client CLI Entrypoint Evidence And Gates

Status: Completed
Last updated: 2026-05-17

## Required Gates

```bash
cargo fmt --all -- --check
cargo check -p taru-client-cli --tests
cargo nextest run -p taru-client-cli --no-fail-fast
cargo tree -p taru-client-cli
cargo check --workspace --tests
cargo nextest run --workspace --no-fail-fast
git diff --check
```

## Boundary Evidence To Record

- `crates/taru-client-cli/Cargo.toml` has `license = "Apache-2.0"`.
- `taru-client-cli` depends on `taru-client`, not on `taru-api`,
  `taru-server`, `taru-core`, `taru-streaming`, or `taru-transcode`.
- Streaming request output redacts bearer token values.
- CLI tests use `taru-client::ClientTransport` mocks instead of a live server.

## Closeout Evidence

- `crates/taru-client-cli/Cargo.toml` uses `license = "Apache-2.0"`.
- `crates/taru-client-cli/src/lib.rs` builds a `taru_client::TaruClient` and
  calls SDK methods or SDK streaming request builders.
- `cargo fmt --all -- --check`: passed.
- `cargo check -p taru-client-cli --tests`: passed.
- `cargo nextest run -p taru-client-cli --no-fail-fast`: 5 tests passed.
- `cargo tree -p taru-client-cli`: passed; manual review shows direct Taru
  dependency is `taru-client`, with `taru-client-protocol` only through the SDK
  and no AGPL server/internal Taru crates.
- `cargo check --workspace --tests`: passed.
- `cargo nextest run --workspace --no-fail-fast`: 279 tests passed.
- `git diff --check`: passed with Git CRLF normalization warnings only.
