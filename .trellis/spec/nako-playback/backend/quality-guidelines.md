# Quality Guidelines

Playback planner changes must remain deterministic and side-effect free.

## Required Patterns

- Prefer Direct Play when source and target are compatible. Transcode is a
  fallback or explicit request, not the default media path.
- Keep profile identity stable and include every request fact that changes the
  planning result.
- Model track selection, audio output, HDR/color pipeline, subtitle strategy,
  and HLS output as typed values.
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

## Tests Required

- Unit tests for Direct Play/Remux/Transcode/Denied selection.
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
- Does runtime work stay outside this crate?
