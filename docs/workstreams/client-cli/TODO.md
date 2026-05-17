# Client CLI Entrypoint Task Ledger

Status: Completed
Last updated: 2026-05-17

## Tasks

- [x] CCLI-010 [owner=codex] [scope=docs/workstreams/client-cli]
  Goal: Open the M37 workstream with license boundary, scope, non-goals, and
  closeout gates.
  Validation: `git diff --check`.
  Handoff: Continue with the Apache-2.0 CLI crate.

- [x] CCLI-020 [owner=codex] [deps=CCLI-010] [scope=crates/taru-client-cli]
  Goal: Add a new Apache-2.0 Rust client CLI crate that uses `taru-client` for
  health, library/item/search, source probe, playback decision, session, and
  streaming request-construction commands.
  Validation: `cargo check -p taru-client-cli --tests` passed.
  Handoff: Added `crates/taru-client-cli`; continue with docs and closeout
  evidence.

- [x] CCLI-030 [owner=codex] [deps=CCLI-020] [scope=crates/taru-client-cli]
  Goal: Add focused tests for command parsing, mocked SDK transport requests,
  streaming request output, bearer-token redaction, and manifest dependency
  boundaries.
  Validation: `cargo nextest run -p taru-client-cli --no-fail-fast` passed
  with 5 tests.
  Handoff: `GET /health` remains an unauthenticated preflight; authenticated
  route tests assert bearer injection through `taru-client`.

- [x] CCLI-040 [owner=codex] [deps=CCLI-030] [scope=docs]
  Goal: Update goal, roadmap, API, and local usage docs for the client CLI.
  Validation: `git diff --check` passed.
  Handoff: Keep Flutter/Dart, Web UI, publishing, and full streaming body
  support as follow-ons.

- [x] CCLI-050 [owner=codex] [deps=CCLI-040] [scope=workspace]
  Goal: Close M37 with focused and workspace validation evidence.
  Validation: `cargo fmt --all -- --check`, `cargo check -p taru-client-cli
  --tests`, `cargo nextest run -p taru-client-cli --no-fail-fast`, `cargo tree
  -p taru-client-cli`, `cargo check --workspace --tests`, `cargo nextest run
  --workspace --no-fail-fast`, `git diff --check` passed.
  Handoff: Recommend the next client slice after the CLI proves the boundary.
