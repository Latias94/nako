# Phase 4.3: FFmpeg Command Builder and Remux Session Skeleton

## Goal

Create the first FFmpeg boundary without starting real FFmpeg processes from
the server. This phase makes remux command planning explicit and gives later
process orchestration, HLS, cancellation, and cleanup work a concrete session
model to build on.

## Implemented Shape

### FFmpeg Command Planning

`taru-transcode` now owns FFmpeg command planning types:

- `FfmpegCommandBuilder`
- `FfmpegCommandPlan`
- `RemuxRequest`
- `RemuxContainer`
- `FfmpegOverwritePolicy`

The initial remux builder creates a copy-only FFmpeg command:

```text
ffmpeg -hide_banner -loglevel warning -n -i <input> -map 0 -c copy -f <format> <output>
```

The builder validates that input and output paths are present and rejects
in-place remux requests. It returns a command plan only; it does not spawn a
process.

### Remux Session Skeleton

`taru-transcode` also defines the in-memory session skeleton:

- `TranscodeSessionId`
- `TranscodeSessionKind::{Remux, HlsTranscode}`
- `TranscodeSessionState`
- `TranscodeSession`
- `TranscodeSessionManager`

The current manager can plan remux sessions and validate lifecycle transitions:

```text
planned -> starting -> running -> cancel_requested -> cancelled
planned -> starting -> running -> finished
starting|running|cancel_requested -> failed
```

This is intentionally not durable yet. Persisted transcode sessions, process
handles, cleanup hooks, and runner integration belong to the next phase.

### Server Configuration

`taru-server` now accepts `ffmpeg_path` alongside `ffprobe_path`. The default
is `ffmpeg`, matching command-line installations on developer machines and
containers.

## Non-Goals

- No FFmpeg process spawning yet.
- No HLS playlists or segments yet.
- No HTTP remux/transcode routes yet.
- No persisted transcode session table yet.
- No hardware acceleration detection or device selection yet.
- No remote source staging/cache integration yet.

## Validation

Coverage added or updated for:

- FFmpeg remux command planning without running FFmpeg;
- copy-only stream mapping with `-map 0` and `-c copy`;
- in-place remux rejection;
- remux session lifecycle transitions;
- invalid session transition rejection;
- server config parsing/defaults for `ffmpeg_path`.

Required gates:

```text
cargo fmt --all -- --check
cargo check --workspace
cargo nextest run --workspace
git diff --check
```
