# Public Client API Contract Evidence And Gates

Status: Completed
Last updated: 2026-05-17

## Starting Repro

The starting architectural repro was:

- `taru-client-protocol` only owns system envelopes and pagination metadata.
- `taru-api` owns client-useful browse/playback DTOs, but they embed
  `taru-core`, `taru-streaming`, and `taru-transcode` types.
- `cargo tree -p taru-client-protocol` was dependency-light, but the useful
  public contract had not yet moved there.

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

### Public Browse Gate

```bash
cargo nextest run -p taru-client-protocol --no-fail-fast
cargo nextest run -p taru-api --no-fail-fast
cargo nextest run -p taru-server http::tests::catalog --no-fail-fast
cargo nextest run -p taru-server http::tests::system --no-fail-fast
```

### Public Playback Gate

```bash
cargo nextest run -p taru-server http::tests::playback --no-fail-fast
```

### Broader Closeout Gate

```bash
cargo nextest run --workspace --no-fail-fast
git diff --check
```

## Evidence Anchors

- `docs/adr/0022-keep-public-protocol-crates-permissive-while-server-crates-remain-agpl.md`
- `docs/workstreams/crate-boundary-hardening/`
- `docs/workstreams/public-client-api/DESIGN.md`
- `docs/workstreams/public-client-api/TODO.md`
- `crates/taru-client-protocol`
- `crates/taru-api`
- `crates/taru-server/src/app/catalog.rs`
- `crates/taru-server/src/app/library.rs`
- `crates/taru-server/src/app/playback/*`
- `crates/taru-server/src/http/tests/catalog.rs`
- `crates/taru-server/src/http/tests/playback.rs`
- `crates/taru-server/src/http/tests/system.rs`

## Prompt-To-Artifact Checklist

- Extend `taru-client-protocol` for Flutter/Web/CLI stable DTOs:
  protocol-owned browse and playback DTOs plus dependency tree evidence.
- Clarify Public Client API vs Server Admin/Internal API:
  DESIGN.md scope, TODO non-goals, and evidence notes for intentionally kept
  internal DTOs.
- Design and implement first catalog/library browse surface:
  protocol DTOs and route tests for browse/search/list/detail.
- Keep `taru-api` as AGPL server adapter:
  mapping functions live in `taru-api`; protocol crate does not import server
  internals.
- Migrate first MediaItem/Library/Playback response subset:
  PCA-020 and PCA-030 evidence.
- Retain diagnostics/job/provider internals in server/API:
  non-goals and diff review.
- Validate:
  final gate output recorded before closeout.

## Recorded Evidence

### PCA-010 Scope And Evidence Freeze

- Workstream docs define the M29 public client API problem, target state,
  non-goals, task ledger, gate set, and prompt-to-artifact checklist.

### PCA-020 Public Browse Protocol DTO Slice

- `crates/taru-client-protocol/src/catalog.rs` owns the first stable public
  library, source, item, metadata, probe, graph, search, image, people, genre,
  tag, and collection DTOs.
- Public IDs are serialized as strings instead of `taru-core` ID newtypes.
- `taru-api` owns explicit adapter functions from server/domain records into
  protocol DTOs.
- Catalog, library, and system route tests continue to validate the shipped
  browse/search/list/detail JSON behavior.

### PCA-030 Public Playback Decision DTO Slice

- `taru-client-protocol` owns public playback decision, direct-play,
  transcode-plan summary, playback mode, output container, and hardware
  acceleration DTOs.
- `taru-api` maps `taru_streaming` and `taru_transcode` planning records into
  protocol DTOs.
- Playback route tests continue to validate the decision and direct stream
  behavior without exposing `taru_streaming::PlaybackDecision` as the public
  response type.

### PCA-040 Contract Docs And Route Evidence

- Public browse/search/list/detail/probe/playback route surfaces are covered by
  the catalog, system, playback, API, and protocol tests listed below.
- Server-admin diagnostics, jobs, provider runtime state, webhook, automation,
  addon administration, ingestion failures, and metadata maintenance DTOs are
  intentionally left in `taru-api` for future demand-driven migration.

### PCA-050 Closeout Validation

- `cargo fmt --all -- --check`: passed.
- `cargo check --workspace --tests`: passed.
- `cargo nextest run -p taru-client-protocol --no-fail-fast`: 3 tests passed.
- `cargo nextest run -p taru-api --no-fail-fast`: 4 tests passed.
- `cargo nextest run -p taru-server http::tests::playback --no-fail-fast`: 16 tests passed.
- `cargo nextest run --workspace --no-fail-fast`: 253 tests passed.
- `cargo tree -p taru-client-protocol`: only normal `serde` and dev
  `serde_json` dependencies; no `taru-core`, `taru-streaming`,
  `taru-transcode`, or `taru-server`.
- `git diff --check`: passed with Git CRLF normalization warnings only.
