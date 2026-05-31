# Transcode Interface And Runtime Plan Deepening - Closeout

Date: 2026-05-31
Status: Closed

## Result

This workstream met its target state. `nako-transcode` now owns the HLS runtime
plan and the FFmpeg execution planning Interface used by server playback HLS
and remux paths. `nako-server` consumes high-level runtime/execution plans
instead of constructing raw `HlsRequest`, `RemuxRequest`,
`FfmpegCommandBuilder`, `FfmpegArg`, or overwrite-policy details.

## Accepted Gates

- `cargo nextest run -p nako-transcode hls --no-fail-fast`
- `cargo nextest run -p nako-transcode remux --no-fail-fast`
- `cargo nextest run -p nako-server hls --no-fail-fast`
- `cargo fmt --all -- --check`
- `git diff --check`

Planner fresh verification for `TIRP-030` passed before merge. Closeout also
validates both affected `WORKSTREAM.json` files and the updated architecture
indexes.

## HDR Unblock

HDR `HTP-030` is no longer blocked by this lane. It should start from current
`main` and keep the implementation inside the transcode-owned runtime and
execution planner Interfaces introduced here.

## Follow-ons

- HDR `HTP-030`: software-first HLS HDR-to-SDR command planning.
- Stage-aware hardware capability matrix and tone-map diagnostics.
- HLS artifact lifecycle consolidation.
- Playback/transcode resource admission unification.
- Broader FFmpeg decoder, encoder, filter, bitstream-filter, and driver quirk
  capability expansion.

## Residual Risks

The shipped Interface deepening reduces server-side assembly, but it does not
solve lifecycle ownership, resource admission, hardware tone mapping, or broad
operator hardware diagnostics. Those should remain explicit workstreams rather
than opportunistic extensions of HDR implementation.
