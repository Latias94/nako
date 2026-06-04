# Control Plane Scan Trace Context

## Goal

Extend the recently shipped request/trace-context baseline beyond HLS by
propagating a redaction-safe trace identity through one library scan or durable
job path.

## Requirements

* Keep ADR 0053 as the control-plane baseline.
* Pick one high-value path only: library scan enqueue/execution or durable job
  execution diagnostics.
* Propagate normalized request identity or trace context without exposing local
  paths, source locators, tokens, raw provider payloads, or FFmpeg command
  lines.
* Keep changes inside `nako-server` unless tests prove a protocol or API
  boundary is required.
* Do not add raw background work or hidden `tokio::spawn` behavior.

## Acceptance Criteria

* [ ] One scan/job path carries typed trace identity across the app boundary.
* [ ] Tests cover propagation and redaction behavior.
* [ ] Existing job runtime and scan behavior remains compatible.
* [ ] No public/admin DTO expansion unless this PRD is explicitly revised.
* [ ] Focused server checks pass.

## Definition Of Done

* `cargo check -p nako-server --tests`
* Focused `cargo nextest run -p nako-server jobs --no-fail-fast` or the closest
  available scan/job test filter.
* `cargo fmt --all -- --check`
* `git diff --check`

## Technical Approach

Start with the existing HTTP trace-context and HLS propagation pattern, then
apply the same redaction discipline to one control-plane path. Keep the slice
small enough that future VFS, FFmpeg, addon, and broader job propagation can
follow without requiring this task to solve every runtime.

## Out Of Scope

* Full incident bundle export.
* Admin diagnostics DTO expansion.
* VFS/FFmpeg/addon trace propagation.
* API scale/cache contract work.
* Raw tracing of sensitive source or provider data.

## Technical Notes

* Lane: `control-plane`.
* Authorized write scope:
  * `crates/nako-server/src/app/jobs.rs`
  * `crates/nako-server/src/app/job_runtime.rs`
  * `crates/nako-server/src/app/library.rs`
  * `crates/nako-server/src/app/metadata_scan.rs`
  * `crates/nako-server/src/app/library_reconciliation.rs`
  * `crates/nako-server/src/app/tests/**`
  * `docs/architecture/CONTROL_PLANE.md`
* Forbidden scope:
  * `crates/nako-server/src/app/playback/**`
  * `apps/admin-web/**`
  * generated contracts

