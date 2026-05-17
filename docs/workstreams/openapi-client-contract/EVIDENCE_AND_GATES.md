# OpenAPI And Public Client SDK Contract Evidence And Gates

Status: Completed
Last updated: 2026-05-17

## Starting Repro

- No OpenAPI generator or artifact exists.
- Human-readable HTTP docs define the public route set, but clients cannot use
  those docs for schema-driven generation.
- Playback session responses currently include a server-local `output_path`.
- Public/auth/error/version semantics exist from M29-M31 but are not expressed
  in a machine-readable contract.

## Gate Set

### Targeted Iteration Gate

```bash
cargo fmt --all -- --check
cargo check --workspace --tests
```

### Protocol Direction Gate

```bash
cargo tree -p taru-client-protocol
```

This must not show dependencies on `taru-core`, `taru-streaming`,
`taru-transcode`, or `taru-server`.

### OpenAPI Focus Gate

```bash
cargo nextest run -p taru-client-protocol --no-fail-fast
cargo nextest run -p taru-api --no-fail-fast
cargo nextest run -p taru-server http::tests --no-fail-fast
```

### Broader Closeout Gate

```bash
cargo nextest run --workspace --no-fail-fast
git diff --check
```

## Evidence Anchors

- `docs/adr/0025-openapi-public-client-sdk-contract.md`
- `docs/api/HTTP_API.md`
- `crates/taru-client-protocol/src/lib.rs`
- `crates/taru-client-protocol/src/catalog.rs`
- `crates/taru-api/src/lib.rs`
- `crates/taru-api/src/openapi.rs`
- `crates/taru-server/src/http/playback.rs`
- `crates/taru-server/src/http/tests`

## Prompt-To-Artifact Checklist

- Establish OpenAPI/Public Client SDK boundary:
  ADR 0025 and workstream docs.
- Keep protocol crate dependency-light:
  `cargo tree -p taru-client-protocol`.
- Move public wire response shapes out of internal/server records:
  playback session DTO cleanup.
- Express auth/version/error/pagination in OpenAPI:
  OpenAPI generator and checker tests.
- Cover public client route set:
  OpenAPI path inventory plus HTTP route tests.
- Reject leakage:
  tests scan OpenAPI for internal crate names, admin route groups, secret
  references, raw provider cache, job internals, and local path fields.
- Validate:
  final gate output recorded before closeout.

## Recorded Evidence

### OAS-010 Scope And Boundary Baseline

- ADR 0025 records the OpenAPI/Public Client SDK contract boundary decision.
- Workstream docs define target state, route scope, non-goals, task ledger,
  evidence anchors, and gate set.

### OAS-020 Protocol Response Hygiene Slice

- `taru-client-protocol` owns `TranscodeSessionResponse`,
  `TranscodeSessionDto`, and client-facing transcode session enums.
- `taru-api::transcode_session_response_from_record` adapts internal
  `TranscodeSessionRecord` values to protocol DTOs.
- Public playback session JSON no longer includes `output_path`.
- Focus validation passed:
  - `cargo nextest run -p taru-client-protocol --no-fail-fast`: 5 tests.
  - `cargo nextest run -p taru-api --no-fail-fast`: 7 tests.
  - `cargo nextest run -p taru-server http::tests::playback http::tests::system --no-fail-fast`: 20 tests.

### OAS-030 OpenAPI Artifact Slice

- `crates/taru-api/src/openapi.rs` generates the Public Client API OpenAPI v1
  document.
- `crates/taru-api/examples/emit-openapi.rs` emits the JSON artifact with
  `cargo run -p taru-api --example emit-openapi`.
- Checker tests verify public route inventory, bearer auth, API-version
  headers, pagination parameters, common error responses, and internal/admin
  leakage rejection.
- `cargo check -p taru-api --examples`: passed.

### OAS-040 Server Route Contract Evidence Slice

- `GET /libraries/{library_id}` returns a protocol-owned `LibraryResponse`.
- `docs/api/HTTP_API.md` lists the OpenAPI generation command and public route
  inventory.
- `cargo nextest run -p taru-server http::tests --no-fail-fast`: passed, 35
  tests.

### OAS-050 Closeout

Prompt-to-artifact audit:

- Establish reusable HTTP API schema foundation:
  `taru_api::openapi::public_openapi_v1()` and
  `taru_api::openapi::public_openapi_v1_json()`.
- Preserve crate boundaries:
  protocol DTOs in `taru-client-protocol`, OpenAPI aggregation in `taru-api`,
  route behavior in `taru-server` tests.
- Produce first OpenAPI v1 artifact:
  `cargo run -p taru-api --example emit-openapi`.
- Cover public client routes:
  checker asserts the exact public path inventory for health, libraries,
  catalog browse/search, source probe, playback decision, direct/remux/HLS,
  playback sessions, cancellation, and HLS segments.
- Express auth/version/error/pagination:
  checker asserts bearer auth, `x-taru-api-version`, `ErrorResponse`,
  `401 unauthorized`, and `limit`/`offset` parameters.
- Reject leakage:
  checker rejects internal crate names, admin-only route groups, secret
  references, raw provider cache, job internals, and local output path fields.
- Update docs:
  ADR 0025, HTTP API docs, goal map, roadmap, and workstream docs.

Closeout validation:

- `cargo fmt --all -- --check`: passed.
- `cargo check --workspace --tests`: passed.
- `cargo nextest run -p taru-client-protocol --no-fail-fast`: passed, 5 tests.
- `cargo nextest run -p taru-api --no-fail-fast`: passed, 7 tests.
- `cargo nextest run -p taru-server http::tests --no-fail-fast`: passed, 35
  tests.
- `cargo check -p taru-api --examples`: passed.
- `cargo nextest run --workspace --no-fail-fast`: passed, 260 tests.
- `cargo tree -p taru-client-protocol`: passed; only `serde` is a normal
  dependency and `serde_json` is dev-only.
- `git diff --check`: passed with Git CRLF normalization warnings only.
