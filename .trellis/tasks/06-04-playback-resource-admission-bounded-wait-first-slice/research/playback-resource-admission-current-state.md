# Research: playback resource admission current state

- Query: identify a feature-backed architecture slice for playback admission
  queueing/waitlist follow-ons.
- Scope: internal
- Date: 2026-06-04

## Findings

* `PlaybackRuntimeAdmission` already owns resource classes, capacity decisions,
  immediate acquisition, bounded `try_acquire_until`, and a typed
  `PlaybackResourceAdmissionPolicy`.
* `PlaybackResourceAdmissionPolicy::HlsSupersede` centralizes the existing HLS
  supersede bounded wait. It rejects unconfigured capacity before waiting and
  keeps timeout/retry constants in `resource.rs`.
* HLS playlist startup currently uses `HlsSupersede` only when older HLS
  sessions are superseded. Ordinary HLS startup stages FFmpeg input and then
  uses `Immediate` admission.
* Direct Play remote stream admission is intentionally non-blocking through a
  separate storage/backend permit path and must not be changed here.
* Archived `06-04-playback-hls-admission-policy-seam` explicitly left durable
  queue/waitlist behavior out of scope, making a bounded HLS start wait the next
  smallest resource-admission deepening.
* HLS seek/restart command identity is already complete and archived, so it is
  not a valid next implementation target.

## Recommended Slice

Add `PlaybackResourceAdmissionPolicy::HlsStart` and route ordinary HLS startup
through that policy. Keep it bounded and process-local; do not add durable queue
state or public API behavior.

## Write Scope

* `crates/nako-server/src/app/playback/resource.rs`
* `crates/nako-server/src/app/playback/hls.rs`
* `crates/nako-server/src/app/playback/hls_flow.rs`
* `crates/nako-server/src/app/tests/playback.rs`
* `.trellis/spec/nako-server/backend/quality-guidelines.md`
* `docs/architecture/PLAYBACK.md`

## Guardrails

* Do not change Direct Play remote stream admission.
* Do not add public route or DTO fields.
* Do not hold staged input while waiting when capacity is known invalid.
* Keep durable queues, priorities, fairness, remote workers, and Admin
  diagnostics for a separate follow-on.
