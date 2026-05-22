# Phase 4.4: Remux Process Runner and Runtime Resource Guard

## Goal

Connect the remux session skeleton to a real FFmpeg process runner while
keeping playback routes unchanged. This phase focuses on process lifecycle,
cancel safety, timeouts, concurrency budgets, and temporary output cleanup
before exposing remux playback through HTTP.

## Implemented Shape

### Remux Runtime Guard

`nako-transcode` now owns remux runtime limits and guards:

- `RemuxRuntimeLimits`
- `RemuxRuntimeGuard`
- `RemuxRuntimePermit`

The guard uses a semaphore to bound concurrent remux sessions. The default is
one concurrent remux and a 30-minute timeout. These values are conservative for
self-hosted systems and can be wired to stronger hardware profiles later.

### Cancellation and Timeout

`CancellationToken` gives the runner an explicit orchestration boundary for
cancel requests. The runner kills the child process on cancellation or timeout,
then removes the temporary output and updates the session state.

Timeouts currently fail the session. Cancellation marks the session cancelled.

### FFmpeg Remux Runner

`FfmpegRemuxRunner` executes a planned remux session:

1. acquire the remux runtime permit;
2. move the session through `starting` and `running`;
3. rewrite the planned final output path to a session-scoped temporary output;
4. spawn the FFmpeg command with null stdin and piped stderr;
5. promote the temporary file to the final output on success;
6. remove temporary output on failure, cancellation, or timeout;
7. update the in-memory session state.

The command still comes from `FfmpegCommandBuilder`, so the runner does not
own codec/container policy.

### Server Runtime Configuration

`nako-server` now accepts:

```toml
remux_concurrency = 1
remux_timeout_ms = 1800000
```

No HTTP route uses the runner yet. Runtime configuration is present so the
next phase can wire remux orchestration without changing the public config
shape again.

## Non-Goals

- No HTTP playback/remux route yet.
- No HLS playlist or segment output yet.
- No persisted transcode session table yet.
- No hardware acceleration detection yet.
- No remote source staging/cache integration yet.
- No process progress parsing yet.

## Validation

Coverage added or updated for:

- successful remux process output promotion;
- failed remux process temporary output cleanup;
- cancelled remux process kill and cleanup;
- timed-out remux process kill, failure state, and cleanup;
- semaphore-based remux concurrency guard;
- server config parsing/defaults for remux runtime budgets.

Required gates:

```text
cargo fmt --all -- --check
cargo check --workspace
cargo nextest run --workspace
git diff --check
```
