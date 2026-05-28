# Executable HLS fMP4 Runtime Boundary TODO

Status: Completed
Last updated: 2026-05-28

## Task Ledger

### EHFR-010 - Open workstream and freeze runtime slice

Status: Completed
Owner: codex
Depends on: none

Scope:

- Create durable workstream docs.
- Link the lane from the workstream index.
- Record fMP4 single-variant as the executable slice and adaptive ladders as a
  follow-on.

Validation:

```text
python3 -m json.tool docs/workstreams/executable-hls-fmp4-runtime-boundary/WORKSTREAM.json
git diff --check -- docs/workstreams/executable-hls-fmp4-runtime-boundary docs/workstreams/README.md
```

### EHFR-020 - Carry HLS output requirement into runtime request identity

Status: Completed
Owner: codex
Depends on: EHFR-010

Scope:

- Thread `HlsOutputRequirement` from playback decision to HLS source runtime.
- Include segment container and variant policy in HLS transcode request identity.
- Keep adaptive policy explicit but non-executable in this first runtime slice.

Validation:

```text
cargo nextest run -p nako-playback --no-fail-fast
cargo nextest run -p nako-server playback --no-fail-fast
```

### EHFR-030 - Add fMP4 staging layout and FFmpeg muxer planning

Status: Completed
Owner: codex
Depends on: EHFR-020

Scope:

- Make HLS staging layout choose `.ts` or `.m4s` segment patterns from the output
  requirement.
- Add fMP4 HLS muxer flags and init segment path planning in `nako-transcode`.
- Preserve current MPEG-TS command output by default.

Validation:

```text
cargo nextest run -p nako-transcode ffmpeg --no-fail-fast
cargo nextest run -p nako-server playback --no-fail-fast
```

### EHFR-040 - Serve fMP4 HLS artifacts safely

Status: Completed
Owner: codex
Depends on: EHFR-030

Scope:

- Serve `.m4s` and init segments with appropriate content types.
- Keep segment-name validation and cleanup safe for both `.ts` and `.m4s`.
- Preserve playlist rewrite and ticket behavior.

Validation:

```text
cargo nextest run -p nako-server hls --no-fail-fast
```

### EHFR-050 - Verify, document evidence, and close first slice

Status: Completed
Owner: codex
Depends on: EHFR-040

Scope:

- Run fresh focused gates and non-test checks.
- Update evidence, handoff, milestones, and closeout docs.
- Split adaptive ladder runtime work into a follow-on.

Validation:

```text
cargo fmt --all -- --check
git diff --check
```
