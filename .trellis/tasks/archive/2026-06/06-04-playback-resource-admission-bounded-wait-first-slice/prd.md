# Playback resource admission bounded wait first slice

## Goal

Move one high-value playback pressure path from immediate denial toward a typed
bounded wait policy: HLS ordinary startup should use a resource-owned
`HlsStart` admission policy instead of open-coded immediate acquisition after
FFmpeg input staging.

## Background

`PlaybackRuntimeAdmission` already owns typed resource classes, capacity
decisions, immediate acquisition, and a bounded wait policy for HLS supersede.
The follow-on architecture lane still calls out admission queueing/waitlists as
future work. A full durable queue is too broad for this slice, but HLS startup
is a good first product-backed boundary because HLS transcode is expensive and
brief contention should not always become an immediate playback failure.

## Requirements

* Add a typed `HlsStart` admission policy in
  `crates/nako-server/src/app/playback/resource.rs`.
* `HlsStart` must be bounded: finite timeout, finite retry interval, and a
  redaction-safe operation label.
* HLS ordinary startup should acquire resource permits through `HlsStart`.
* HLS supersede must continue to use the existing `HlsSupersede` policy.
* Direct Play remote stream admission must remain non-blocking and unchanged.
* Preserve configured-capacity rejection before waiting.
* Avoid staging FFmpeg input when HLS start resource capacity is already known
  to be unavailable.
* Release acquired permits naturally if FFmpeg input staging fails.
* Update tests for immediate rejection, bounded wait, and HLS flow behavior.
* Do not add public API/DTO/schema changes, durable waitlist persistence,
  remote workers, LL-HLS/CMAF, or player UX changes.

## Technical Approach

Add a new resource policy variant:

* `Immediate`: unchanged, non-waiting acquisition.
* `HlsStart`: short bounded wait for ordinary HLS starts.
* `HlsSupersede`: existing bounded wait used after cancelling older HLS
  generations.

Use `PlaybackRuntimeAdmission::ensure_capacity_for_policy` before staging HLS
input, then acquire the HLS start permit through `acquire_for_policy`. This
keeps HLS input staging from doing work when configured capacity is invalid, and
keeps the wait behavior centralized in `resource.rs`.

## Acceptance Criteria

* [ ] `HlsStart` bounded wait policy exists and is tested.
* [ ] HLS source/playlist ordinary startup uses `HlsStart`.
* [ ] HLS supersede behavior remains on `HlsSupersede`.
* [ ] Direct Play remote stream pressure remains immediate/non-waiting.
* [ ] Focused server playback resource and HLS gates pass.
* [ ] Architecture/spec notes record the first bounded HLS start wait slice and
  keep durable queue/waitlist as a follow-on.

## Verification Plan

* `cargo fmt --all -- --check`
* `cargo check -p nako-server --tests`
* `cargo nextest run -p nako-server playback_resource_admission --no-fail-fast`
* `cargo nextest run -p nako-server hls_source --no-fail-fast`
* `cargo nextest run -p nako-server hls_playlist --no-fail-fast`
* `git diff --check`
* `python ./.trellis/scripts/task.py validate .trellis/tasks/06-04-playback-resource-admission-bounded-wait-first-slice`

## Out Of Scope

* Durable admission queue or persisted waitlist.
* Remote transcode workers.
* Direct Play waiting semantics.
* Public/Admin API changes.
* Schema or repository changes.
* LL-HLS/CMAF and player UX.
