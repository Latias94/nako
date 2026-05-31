# Playback Runtime Resource Scheduler — Evidence And Gates

Status: Completed
Last updated: 2026-05-31

## Smallest Current Repro

The current proof is the PRRS-050 closeout gate for the shipped single-node
playback admission slice.

```bash
cargo nextest run -p nako-server playback --no-fail-fast
cargo nextest run -p nako-server hls --no-fail-fast
```

## Gate Set

### Planning Gate

```bash
python3 -m json.tool docs/workstreams/playback-runtime-resource-scheduler/WORKSTREAM.json
git diff --check -- docs/workstreams/playback-runtime-resource-scheduler docs/architecture/PLAYBACK.md docs/architecture/WORKSTREAM_LINKS.md docs/workstreams/README.md
```

Proves the workstream docs are syntactically valid and do not introduce
whitespace artifacts.

### Admission Model Gate

```bash
cargo nextest run -p nako-server playback_resource --no-fail-fast
cargo nextest run -p nako-server playback --no-fail-fast
```

Proves playback resource demand and admission decisions without relying on a
full route matrix.

### HLS And Remux Enforcement Gate

```bash
cargo nextest run -p nako-server hls --no-fail-fast
cargo nextest run -p nako-server playback --no-fail-fast
```

Proves HLS/remux permit behavior while preserving existing playback and route
contracts.

### Operations Gate

```bash
cargo nextest run -p nako-server admin_v1_playback --no-fail-fast
cargo nextest run -p nako-server playback --no-fail-fast
cargo nextest run -p nako-api --no-fail-fast
```

Proves Admin diagnostics and runtime settings report scheduler pressure without
leaking unsafe details. The `nako-api` gate is required when Admin DTOs or
generated web contracts change.

### Final Closeout Gate

```bash
cargo nextest run -p nako-server playback --no-fail-fast
cargo nextest run -p nako-server hls --no-fail-fast
cargo fmt --all -- --check
git diff --check
```

Use broader workspace gates only if this lane changes shared API crates or
wire contracts outside focused playback/runtime surfaces.

### Review Gate

Run `review-workstream` before accepting task or lane completion. Record
blocking findings, missing gates, and residual risks here or link to a review
note.

## Evidence Log

| Date | Task | Evidence | Status | Notes |
| --- | --- | --- | --- | --- |
| 2026-05-29 | PRRS-010 | Workstream opened | Passed | `python3 -m json.tool docs/workstreams/playback-runtime-resource-scheduler/WORKSTREAM.json`; `git diff --check -- docs/workstreams/playback-runtime-resource-scheduler docs/architecture/PLAYBACK.md docs/architecture/WORKSTREAM_LINKS.md docs/workstreams/README.md`. |
| 2026-05-29 | PRRS-020 | Playback resource admission model | Passed | Added `crates/nako-server/src/app/playback/resource.rs` with direct/remux/HLS demand classes and decision statuses for accepted, rejected, and not-yet-enforced resources. Fresh gates: `cargo nextest run -p nako-server playback_resource --no-fail-fast`; `cargo nextest run -p nako-server playback --no-fail-fast`; `cargo fmt --all -- --check`; `python3 -m json.tool docs/workstreams/playback-runtime-resource-scheduler/WORKSTREAM.json`; `git diff --check`. `review-workstream` self-review found no blocking or important findings; remaining permit-lifetime wiring is PRRS-030 scope. Full workspace test was not run because PRRS-020 is scoped to `nako-server` playback internals and workstream docs. |
| 2026-05-29 | PRRS-030 | HLS/remux admission permit enforcement | Passed | Added host-owned admission semaphores and wired permit acquisition into HLS/remux start paths, including browser playback preflight paths. Fresh gates: `cargo nextest run -p nako-server hls --no-fail-fast` (52 passed, 407 skipped); `cargo nextest run -p nako-server playback --no-fail-fast` (130 passed, 329 skipped); `cargo fmt --all -- --check`; `python3 -m json.tool docs/workstreams/playback-runtime-resource-scheduler/WORKSTREAM.json`; `git diff --check`. `review-workstream` self-review found no blocking or important findings after preserving original admission errors on HLS source-input release failures. Full workspace test was not run because PRRS-030 is scoped to `nako-server` playback runtime paths and workstream docs. |
| 2026-05-29 | PRRS-040 | Admin runtime pressure diagnostics | Passed | Added redaction-safe `resource_pressure` Admin runtime diagnostics with configured capacity, available permits, in-use permits, resource class, and enforcement mode. Fresh gates: `cargo nextest run -p nako-server admin_v1_playback --no-fail-fast` (10 passed, 453 skipped); `cargo nextest run -p nako-server playback --no-fail-fast` (132 passed, 331 skipped); `cargo nextest run -p nako-api --no-fail-fast` (69 passed); `cargo fmt --all -- --check`; `python3 -m json.tool docs/workstreams/playback-runtime-resource-scheduler/WORKSTREAM.json`; `git diff --check`. `review-workstream` self-review found no blocking or important findings. Full workspace test was not run because PRRS-040 is scoped to Admin playback diagnostics, playback runtime paths, and Admin API contracts. |
| 2026-05-29 | PRRS-050 | Closeout and follow-on split | Passed | Closeout docs mark the single-node first slice complete and split queueing, remote transcode workers, OS resource isolation, per-device tuning, LL-HLS/DASH, DRM/key delivery, and HLS artifact I/O pressure enforcement into follow-ons. Fresh gates: `cargo nextest run -p nako-server playback_resource --no-fail-fast` (6 passed, 457 skipped); `cargo nextest run -p nako-server playback --no-fail-fast` (132 passed, 331 skipped); `cargo nextest run -p nako-server hls --no-fail-fast` (53 passed, 410 skipped); `cargo nextest run -p nako-server admin_v1_playback --no-fail-fast` (10 passed, 1 leaky, 453 skipped); `cargo nextest run -p nako-api --no-fail-fast` (69 passed); `cargo fmt --all -- --check`; `python3 -m json.tool docs/workstreams/playback-runtime-resource-scheduler/WORKSTREAM.json`; `git diff --check`. `review-workstream`/`verify-rust-workstream` closeout review found no blocking or important findings after correcting the design target to treat queueing and HLS artifact I/O enforcement as follow-ons. |
| 2026-05-31 | PRRS post-closeout | Runtime guard/admission ownership cleanup | Passed | Server HLS/remux composition now uses `TranscodeRuntimeGuard::timeout_only`, leaving process concurrency ownership with `PlaybackRuntimeAdmission` while preserving `TranscodeRuntimeGuard::new` as the bounded low-level runner guard for direct transcode tests/adapters. Fresh gates: `cargo nextest run -p nako-transcode remux_runtime_guard_bounds_concurrent_sessions --no-fail-fast` (1 passed); `cargo nextest run -p nako-server playback_resource --no-fail-fast` (6 passed, 499 skipped); `cargo nextest run -p nako-server remux --no-fail-fast` (17 passed, 488 skipped); `cargo nextest run -p nako-transcode remux --no-fail-fast` (14 passed, 84 skipped); `cargo nextest run -p nako-server hls --no-fail-fast --test-threads=1` (71 passed, 434 skipped); `cargo check -p nako-server -p nako-transcode` passed with only existing repository unused/dead-code warnings outside this change. First default-parallel `cargo nextest run -p nako-server hls --no-fail-fast` passed 69 tests and timed out 2 ready-before-runner-exit tests; both timed-out tests passed individually and the serial full HLS gate passed. |

## Evidence Anchors

- `docs/workstreams/playback-runtime-resource-scheduler/DESIGN.md`
- `docs/workstreams/playback-runtime-resource-scheduler/TODO.md`
- `docs/workstreams/playback-runtime-resource-scheduler/MILESTONES.md`
- `crates/nako-server/src/app/playback`
- `crates/nako-server/src/config.rs`
- `crates/nako-server/src/app.rs`
- `crates/nako-server/src/http/admin.rs`
- `crates/nako-transcode/src/runtime.rs`
- `docs/workstreams/playback-runtime-resource-scheduler/CLOSEOUT.md`

Fresh verification is required before marking a task, Codex goal, or lane
complete.
