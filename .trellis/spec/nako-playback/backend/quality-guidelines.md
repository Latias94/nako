# Quality Guidelines

Playback planner changes must remain deterministic and side-effect free.

## Required Patterns

- Prefer Direct Play when source and target are compatible. Transcode is a
  fallback or explicit request, not the default media path.
- Evaluate selected subtitle delivery before choosing Remux. A container
  fallback must not bypass a subtitle requirement that only HLS transcode can
  satisfy, such as ASS/SSA burn-in.
- Keep profile identity stable and include every request fact that changes the
  planning result.
- Model track selection, audio output, HDR/color pipeline, subtitle strategy,
  and HLS output as typed values.
- Treat unknown subtitle codec facts as an explicit policy choice. If unknown
  codecs preserve a legacy sidecar path, cover that behavior with a named test
  and document it rather than relying on implicit `None` handling.
- Treat sidecar-capable subtitle codec facts as a mixed vocabulary. Probe facts
  may carry FFmpeg codec names such as `webvtt` or sidecar extension aliases
  such as `vtt`; both must map to sidecar delivery when the client supports
  subtitles.
- Keep `PlaybackDecisionReport` useful even when playback is denied.
- Keep storage facts abstract: remote/range-readable are planning inputs, not
  backend calls.

## Forbidden Patterns

- Do not add process execution, filesystem staging, HTTP serving, or database
  writes to this crate.
- Do not make Source Variant Labels decide compatibility; use Media Technical
  Facts, client capabilities, and policy.
- Do not assume all clients support HLS TS/fMP4, AAC, H264, HDR, subtitles, or
  range requests.
- Do not hide policy denial by selecting a fallback mode the policy disallows.
- Do not select Remux before checking whether the selected subtitle track
  requires burn-in or another transcode-only delivery strategy.

## Tests Required

- Unit tests for Direct Play/Remux/Transcode/Denied selection.
- Regression tests for container-unsupported/remux-supported sources with
  selected subtitle tracks that require burn-in.
- Tests that name the chosen behavior for missing or blank subtitle codec facts.
- Tests for sidecar subtitle codec aliases when changing HLS sidecar-versus-
  burn-in classification, including both codec-name and file-extension forms
  such as `webvtt` and `vtt`.
- Tests for profile identity changes when request facts change.
- Tests for audio downmix/normalization and HDR/color pipeline requirements
  when those values change.
- Server integration tests when resource admission or HTTP route behavior
  changes.

## Gate Selection

- Focused planner:
  `cargo nextest run -p nako-playback <filter> --no-fail-fast`
- Playback/server cross-crate:
  `cargo check -p nako-playback -p nako-transcode -p nako-server --tests`

## Review Checklist

- Is the planner still pure?
- Are every decision reason and denial testable?
- Are new client/source facts included in profile identity?
- Can selected subtitle delivery change the result before Remux is selected?
- Does runtime work stay outside this crate?

## Wrong vs Correct

### Wrong

```rust
if report.remux.supported {
    return PlaybackMode::Remux;
}

let subtitle_requirement = selected_subtitle_requirement(...);
```

This can let a remux-capable container bypass a selected subtitle that requires
burn-in.

### Correct

```rust
let subtitle_requirement = selected_subtitle_requirement(...);
let report = evaluate_remux(..., subtitle_requirement);

if report.remux.supported {
    return PlaybackMode::Remux;
}
```

Subtitle delivery is part of compatibility, not a later decoration.
