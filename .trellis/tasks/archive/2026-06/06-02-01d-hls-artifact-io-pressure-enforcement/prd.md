# HLS Artifact IO Pressure Enforcement

## Goal

Add bounded artifact I/O pressure policy for HLS/runtime artifacts inside the
playback-transcode lane.

## Requirements

* Preserve ADR 0052 HLS runtime and media engine boundaries.
* Treat artifact I/O pressure as playback/transcode runtime policy, not VFS
  cache repair.
* Avoid raw unbounded background work; follow bounded async/resource budget
  decisions.
* Keep Public/Admin API changes out of the MVP unless diagnostics require a
  small read-only surface.
* Add tests around pressure classification, admission/deferral behavior, and
  failure reporting.

## Acceptance Criteria

* [ ] HLS artifact work has an explicit resource/pressure policy.
* [ ] Runtime behavior remains bounded under concurrent sessions.
* [ ] Tests cover normal, pressured, and rejected/deferred paths.
* [ ] Playback architecture docs or Trellis specs are updated if the invariant
      changes.

## Definition of Done

* Scoped nextest run passes for playback/transcode/server areas touched.
* No VFS cache repair policy is mixed into the task.
* No UI work unless added through a follow-on.

## Worktree

Suggested path: `E:\Rust\nako-worktrees\01d-hls-artifact-io-pressure`

Suggested branch: `task/01d-hls-artifact-io-pressure`

Conflict note: coordinate with storage work only where playback input staging or
source reads cross VFS boundaries.
