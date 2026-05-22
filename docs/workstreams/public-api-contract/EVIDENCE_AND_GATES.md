# Public API Contract Hardening Evidence And Gates

Status: Completed
Last updated: 2026-05-17

## Starting Repro

The current architectural repro is:

- `nako-client-protocol` owns `ErrorResponse`, but only as `code/message`
  strings without a protocol-owned vocabulary.
- `nako-server/src/http/error.rs` owns the effective error code and status
  mapping.
- `/health` reports `API_VERSION`, but version compatibility rules are not
  documented beyond the current response field.
- Public client routes and server-admin/internal routes share implementation
  machinery, but their compatibility promises are not explicitly separated.

## Gate Set

### Targeted Iteration Gate

```bash
cargo fmt --all -- --check
cargo check --workspace --tests
```

### Protocol Direction Gate

```bash
cargo tree -p nako-client-protocol
```

This must not show dependencies on `nako-core`, `nako-streaming`,
`nako-transcode`, or `nako-server`.

### Public Contract Focus Gate

```bash
cargo nextest run -p nako-client-protocol --no-fail-fast
cargo nextest run -p nako-api --no-fail-fast
cargo nextest run -p nako-server http::tests::system --no-fail-fast
cargo nextest run -p nako-server http::tests::catalog --no-fail-fast
cargo nextest run -p nako-server http::tests::playback --no-fail-fast
```

### Broader Closeout Gate

```bash
cargo nextest run -p nako-server http::tests --no-fail-fast
cargo nextest run --workspace --no-fail-fast
git diff --check
```

## Evidence Anchors

- `docs/adr/0023-public-api-versioning-and-error-envelope-contract.md`
- `docs/api/HTTP_API.md`
- `docs/workstreams/public-api-contract/DESIGN.md`
- `docs/workstreams/public-api-contract/TODO.md`
- `crates/nako-client-protocol/src/lib.rs`
- `crates/nako-api/src/lib.rs`
- `crates/nako-server/src/http/error.rs`
- `crates/nako-server/src/http/query.rs`
- `crates/nako-server/src/http/system.rs`
- `crates/nako-server/src/http/tests/catalog.rs`
- `crates/nako-server/src/http/tests/playback.rs`
- `crates/nako-server/src/http/tests/system.rs`

## Prompt-To-Artifact Checklist

- Stabilize HTTP API version, error response, pagination/response envelope,
  and compatibility rules:
  ADR 0023, DESIGN.md, HTTP_API.md, protocol tests, and route tests.
- Clarify Public Client API vs Server Admin/Internal API:
  DESIGN.md scope, HTTP_API.md route boundary, and TODO non-goals.
- Make public catalog/library/playback/system route behavior testable:
  focused route tests in `crates/nako-server/src/http/tests`.
- Audit current `nako-api` / `nako-server` behavior:
  Starting Audit in DESIGN.md and Starting Repro in this file.
- Preserve out-of-scope admin/internal migration:
  TODO and DESIGN non-goals.
- Validate:
  final gate output recorded before closeout.

## Recorded Evidence

### PAC-010 Scope And Contract Baseline

- Workstream docs define the M30 public API versioning/error-envelope problem,
  target state, non-goals, task ledger, gate set, and prompt-to-artifact
  checklist.
- ADR 0023 records the public API versioning and error envelope decision.

### PAC-020 Protocol Error Vocabulary Slice

- `crates/nako-client-protocol/src/lib.rs` owns `ClientErrorCode`, its stable
  v1 wire values, `ErrorResponse::new`, and `API_VERSION_HEADER`.
- `crates/nako-api/src/lib.rs` re-exports the protocol error vocabulary and
  API version header constant for server adapter use.
- `nako-client-protocol` tests prove error code serialization and lookup while
  keeping `ErrorResponse` JSON as `code/message`.

### PAC-030 Server Error Mapping And Version Identity Slice

- `crates/nako-server/src/http/error.rs` maps `NakoError` to
  `ClientErrorCode` instead of free string literals.
- `crates/nako-server/src/http.rs` adds `x-nako-api-version: v1` to HTTP
  responses.
- `crates/nako-server/src/http/tests/system.rs` verifies `/health.version`,
  the version response header, pagination metadata for `/libraries`, stable
  public error codes, safe database messages, and code lookup through
  `ClientErrorCode`.

### PAC-040 Public Route Contract Evidence Slice

- `crates/nako-server/src/http/tests/catalog.rs` verifies public catalog/list
  pagination metadata, including `limit`, `offset`, and `returned`.
- `crates/nako-server/src/http/tests/playback.rs` verifies public playback
  not-found and invalid-pagination error codes through protocol-owned
  `ClientErrorCode`.
- `docs/api/HTTP_API.md` documents public API v1 version identity, the
  `x-nako-api-version` response header, the public route subset,
  public/internal boundary, pagination rules, and protocol-owned error codes.

### PAC-050 Closeout Validation

- `cargo fmt --all -- --check`: passed.
- `cargo check --workspace --tests`: passed.
- `cargo nextest run -p nako-client-protocol --no-fail-fast`: 4 tests passed.
- `cargo nextest run -p nako-api --no-fail-fast`: 4 tests passed.
- `cargo nextest run -p nako-server http::tests::system --no-fail-fast`: 3 tests passed.
- `cargo nextest run -p nako-server http::tests::catalog --no-fail-fast`: 3 tests passed.
- `cargo nextest run -p nako-server http::tests::playback --no-fail-fast`: 16 tests passed.
- `cargo nextest run -p nako-server http::tests --no-fail-fast`: 34 tests passed.
- `cargo nextest run --workspace --no-fail-fast`: 254 tests passed.
- `cargo tree -p nako-client-protocol`: only normal `serde` and dev
  `serde_json` dependencies; no `nako-core`, `nako-streaming`,
  `nako-transcode`, or `nako-server`.
