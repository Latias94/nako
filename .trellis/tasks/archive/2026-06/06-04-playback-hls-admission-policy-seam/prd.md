# playback hls admission policy seam

## Goal

Refactor HLS playback resource admission so bounded wait behavior is owned by a
typed resource-admission policy instead of being open-coded in HLS source and
playlist orchestration paths.

## What I Already Know

* Direct Play remote stream pressure must remain non-blocking and return stable
  `409` conflict behavior; this task must not change that contract.
* HLS supersede currently uses the same constants and `try_acquire_until` call
  in both `hls.rs` and `hls_flow.rs`.
* `PlaybackRuntimeAdmission` already owns `try_acquire`, `try_acquire_until`,
  configured-capacity checks, and pressure diagnostics.
* The recent HLS orchestration refactor moved source/playlist flow coordination
  into `hls_flow.rs`, but admission wait policy is still duplicated at the call
  sites.
* ADR 0053 and the `nako-server` specs place resource admission in app runtime
  helpers, not HTTP handlers or pure planner crates.

## Requirements

* Introduce a small typed HLS admission policy seam in
  `crates/nako-server/src/app/playback/resource.rs`.
* Preserve existing behavior for:
  * immediate HLS admission when there are no supersede candidates;
  * bounded HLS supersede wait with the same timeout and retry interval;
  * configured-capacity rejection before waiting;
  * Direct Play remote stream non-blocking admission.
* Replace duplicated HLS supersede admission logic in `hls.rs` and
  `hls_flow.rs` with the resource-owned policy entry point.
* Add or update focused tests around immediate rejection, bounded wait, and
  configured-capacity rejection.
* Do not change public HTTP/API response contracts or schema.

## Acceptance Criteria

* [ ] HLS supersede wait constants and acquire behavior are centralized in a
  typed resource admission policy.
* [ ] HLS source and HLS playlist paths call the same policy entry point.
* [ ] Existing HLS behavior remains unchanged.
* [ ] Focused server playback tests pass.
* [ ] `cargo check -p nako-server --tests` passes.
* [ ] `cargo fmt --all -- --check` and `git diff --check` pass.

## Definition Of Done

* Code is committed with a Conventional Commit message.
* Task evidence records validation commands.
* Relevant Trellis spec is updated if the resource admission policy becomes a
  reusable convention.
* Task is archived after commit and journal is updated.

## Out Of Scope

* No real distributed queue, durable waitlist, or remote worker implementation.
* No Direct Play waiting semantics.
* No public API/DTO/schema migration.
* No LL-HLS/CMAF, subtitle burn-in, hardware tone mapping, or player UX work.

## Technical Approach

Add a resource-owned helper such as `acquire_for_policy` plus a small policy enum
or struct for immediate versus bounded supersede wait. Keep operation labels
redaction-safe and preserve the current five-second HLS supersede wait and
50-millisecond retry cadence. The code change should make future queueing work
attach to one resource-admission seam instead of spreading timeout behavior
through playback orchestration.

## Verification

* `cargo fmt --all -- --check`
* `git diff --check`
* `cargo check -p nako-server --tests`
* `cargo nextest run -p nako-server playback_resource_admission --no-fail-fast`
* `cargo nextest run -p nako-server hls --no-fail-fast`
