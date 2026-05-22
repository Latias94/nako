# Phase 26.0: Playback Client Contract

## Status

Completed.

## Objective

Stabilize the playback/session HTTP contract before real clients depend on it.
The first client-facing control surface is session cancellation for remux and
HLS transcode sessions.

## Scope

- `GET /playback/sessions/{session_id}` remains the inspection route.
- `POST /playback/sessions/{session_id}/cancel` requests cancellation.
- Successful cancellation returns `TranscodeSessionResponse`.
- Missing sessions return `404 not_found`.
- Terminal sessions return `409 conflict`.
- Active records that are not running in the current process return
  `409 conflict`.
- Cancellation is process-local and best-effort in the current single-process
  runtime.

## Implementation Notes

`nako-server::app::playback` owns a process-local cancellation registry keyed
by `TranscodeSessionId`. Remux and HLS orchestration register the runner
`CancellationToken` before invoking FFmpeg and remove it when the run returns.
The HTTP route calls the application service, which signals the live token and
then conditionally moves the persisted row to `cancel_requested`.

The conditional database update prevents a late cancellation request from
moving a terminal session back into an active state.

## Client Contract

Session states are stable strings:

```text
active:   planned, starting, running, cancel_requested
terminal: finished, failed, cancelled
```

Clients should treat `cancel_requested` as an accepted control request and
poll `GET /playback/sessions/{session_id}` until the session becomes
`cancelled`, `failed`, or `finished`.

## Validation

Close-out gates:

- `cargo fmt --all -- --check`
- `cargo check -p nako-server --tests`
- `cargo check --workspace --tests`
- `cargo nextest run -p nako-server http::tests::playback --no-fail-fast`: 16
  playback route tests passed.
- `cargo nextest run --workspace --no-fail-fast`: 234 tests passed.
- `git diff --check`: passed with Git CRLF normalization warnings only.
