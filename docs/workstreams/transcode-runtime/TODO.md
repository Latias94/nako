# Transcode Runtime TODO

## M25.0 Design Baseline

- [ ] Audit `crates/taru-server/src/app/playback.rs` and list the target module
      split.
- [ ] Audit `crates/taru-transcode/src/lib.rs` and identify command planning,
      runner, hardware, and session-manager seams.
- [ ] Define stable playback session lifecycle states and client-visible error
      categories.
- [ ] Decide which diagnostics belong in HTTP APIs and which stay internal.
- [ ] Record validation commands for the first code slice.

## Playback Service Decomposition

- [ ] Split direct-play planning from remux and HLS orchestration.
- [ ] Split remux orchestration into a focused internal service.
- [ ] Split HLS orchestration into a focused internal service.
- [ ] Keep storage staging and manifest coordination explicit.
- [ ] Preserve route behavior and existing API DTOs.

## Hardware Capability Probe

- [ ] Add FFmpeg-backed capability detection behind a testable trait.
- [ ] Detect or infer VAAPI support.
- [ ] Detect or infer NVENC support.
- [ ] Detect or infer QuickSync/QSV support.
- [ ] Preserve CPU fallback and configured fail policy.
- [ ] Add diagnostics without exposing sensitive paths or credentials.

## Runtime Contracts

- [ ] Document session creation, reuse, conflict, running, finished, failed,
      cancelled, and stale-startup recovery behavior.
- [ ] Ensure resource budget selection follows selected acceleration.
- [ ] Add focused tests for timeout, cancellation, fallback, and unavailable
      hardware behavior.
- [ ] Update HTTP API docs for playback session and hardware diagnostics if
      public shapes change.
