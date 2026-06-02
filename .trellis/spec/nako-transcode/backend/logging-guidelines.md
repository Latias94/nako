# Logging Guidelines

Transcode diagnostics should help operators debug FFmpeg behavior without
leaking sensitive machine-local details.

## Rules

- Do not log raw tokens, playback tickets, credentials, or signed URLs.
- Be careful with local paths in logs; prefer session IDs, source IDs, artifact
  IDs, engine kind, codec/container choices, and safe failure categories.
- Use FFmpeg progress parsing and typed runtime summaries over unstructured
  stdout/stderr dumps when exposing diagnostics.
- Hardware probe diagnostics should distinguish unavailable hardware, probe
  error, and unsupported policy.

## Evidence

- `crates/nako-transcode/src/progress.rs`
- `crates/nako-transcode/src/hardware.rs`
- `crates/nako-transcode/src/runtime.rs`
