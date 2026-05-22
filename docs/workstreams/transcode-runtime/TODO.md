# Transcode Runtime TODO

## M25.0 Design Baseline

- [x] Audit `crates/nako-server/src/app/playback.rs` and list the target module
      split.
- [x] Audit `crates/nako-transcode/src/lib.rs` and identify command planning,
      runner, hardware, and session-manager seams.
- [x] Define stable playback session lifecycle states and client-visible error
      categories.
- [x] Decide which diagnostics belong in HTTP APIs and which stay internal.
- [x] Record validation commands for the first code slice.

## Playback Service Decomposition

- [x] Split direct-play planning from remux and HLS orchestration.
- [x] Split remux orchestration into a focused internal service.
- [x] Split HLS orchestration into a focused internal service.
- [x] Keep storage staging and manifest coordination explicit.
- [x] Preserve route behavior and existing API DTOs.

## Hardware Capability Probe

- [x] Add FFmpeg-backed capability detection behind a testable trait.
- [x] Detect or infer VAAPI support.
- [x] Detect or infer NVENC support.
- [x] Detect or infer QuickSync/QSV support.
- [x] Preserve CPU fallback and configured fail policy.
- [x] Add diagnostics without exposing sensitive paths or credentials.

## Runtime Contracts

- [x] Document session creation, reuse, conflict, running, finished, failed,
      cancelled, and stale-startup recovery behavior.
- [x] Ensure resource budget selection follows selected acceleration.
- [x] Add focused tests for timeout, cancellation, fallback, and unavailable
      hardware behavior.
- [x] Update HTTP API docs for playback session and hardware diagnostics if
      public shapes change.
## Post-M25 Follow-Ups

- [ ] Add device-initialization hardware diagnostics if encoder-name probing is
      too weak for operators.
- [ ] Define HLS request identity and future Transcode Profile cache keys
      before changing HLS session reuse or cache semantics.

## M26.0 Playback API Contract and Client Readiness

- [x] Add a public playback session cancellation endpoint.
- [x] Wire cancellation to live remux/HLS runner tokens.
- [x] Return `TranscodeSessionResponse` for session inspection and successful
      cancellation.
- [x] Map missing, terminal, and process-local stale active sessions to stable
      `404 not_found` or `409 conflict` envelopes.
- [x] Document active/terminal playback session states and cancellation
      semantics in the HTTP API guide.
- [x] Add route-level tests for active cancellation, terminal cancellation
      conflict, process-local stale active-session conflict, session
      inspection, and HLS segment readiness/error behavior.
