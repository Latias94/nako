# Quality Guidelines

Transcode changes must keep FFmpeg behavior typed, bounded, and testable.

## Required Patterns

- Build FFmpeg commands through typed request and command builder structs.
- Assert exact argv shape in unit tests for new command-planning behavior.
- Use artifact manifests before publishing HLS/remux outputs to callers.
- Keep overwrite policy explicit.
- Keep hardware acceleration policy and capability reports explicit; do not
  probe GPU support for every playback request.
- Keep runtime limits bounded with concurrency and timeout values.

## Forbidden Patterns

- Do not concatenate FFmpeg command strings by hand.
- Do not publish HLS playlists or segment paths unless a typed manifest proves
  they are generated and safe to expose.
- Do not default to transcode when Direct Play or Remux is compatible; that is
  a playback planner decision.
- Do not assume HLS segment container, codec, or hardware acceleration support
  without a policy/capability fact.
- Do not hide CPU/GPU/disk/network pressure inside the FFmpeg builder.

## Tests Required

- Unit tests for command argv, overwrite policy, in-place output rejection, and
  HLS artifact manifest shape.
- Unit tests for hardware capability and degraded inventory behavior.
- Runtime tests for concurrency limits, timeout, progress, and cancellation when
  those behaviors change.
- Server integration tests when admission, playback route, or artifact serving
  behavior changes.

## Gate Selection

- Focused transcode:
  `cargo nextest run -p nako-transcode <filter> --no-fail-fast`
- Playback/transcode/server:
  `cargo check -p nako-playback -p nako-transcode -p nako-server --tests`

## Review Checklist

- Is every FFmpeg argument typed and tested?
- Are artifacts represented by manifests?
- Are hardware and runtime budgets explicit?
- Does playback selection remain outside this crate?
