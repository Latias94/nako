# Source-Aware Transcode Runtime - Milestones

Status: Completed
Last updated: 2026-05-28

## M0 - Scope And Evidence Freeze

Exit criteria:

- Problem and target state are explicit.
- Non-goals are explicit.
- Relevant ADRs, workstreams, and reference material are linked.
- First executable slice is chosen.

Primary evidence:

- `docs/workstreams/source-aware-transcode-runtime/DESIGN.md`
- `docs/workstreams/source-aware-transcode-runtime/TODO.md`
- `docs/adr/0049-source-aware-transcode-runtime.md`

## M1 - Source Media Technical Facts

Exit criteria:

- `MediaProbeResult` carries source-aware facts needed by playback and
  transcode planning.
- ffprobe parsing maps those facts from representative JSON payloads.
- Existing probe payloads remain compatible.

Primary gates:

- `cargo nextest run -p nako-media-probe --no-fail-fast`
- `cargo nextest run -p nako-core --no-fail-fast`

## M2 - Playback Requirement Deepening

Exit criteria:

- Playback planning returns explicit source-aware transcode requirements.
- Direct/remux/transcode reasons can distinguish codec, profile, bit depth,
  HDR/tone-map, subtitle, audio, and output constraint pressure.
- Public Client surfaces stay stable or have intentional generated updates.

Primary gates:

- `cargo nextest run -p nako-playback --no-fail-fast`
- `cargo nextest run -p nako-api --no-fail-fast`

## M3 - Source-Aware Pipeline And FFmpeg Command Planning

Exit criteria:

- Pipeline planning consumes source facts and output requirements.
- Hardware fallback is typed and stage-specific.
- FFmpeg command construction is split into testable components.

Primary gates:

- `cargo nextest run -p nako-transcode --no-fail-fast`
- `cargo nextest run -p nako-server playback --no-fail-fast`

## M4 - Runtime Supervision And Progressive HLS Foundation

Exit criteria:

- FFmpeg progress can be parsed and surfaced as redaction-safe session metrics.
- HLS runtime can distinguish in-progress and terminal artifacts.
- Segment cleanup/throttle settings are connected to runtime behavior or split
  into explicit follow-ons.

Primary gates:

- `cargo nextest run -p nako-transcode progress --no-fail-fast`
- `cargo nextest run -p nako-server hls --no-fail-fast`

## M5 - Closeout

Exit criteria:

- Gate set is fresh.
- Workstream evidence and handoff reflect shipped behavior.
- Remaining adaptive HLS, rsmpeg, remote worker, or UI work is split into
  follow-ons.
- `WORKSTREAM.json` status is updated.

Status: Complete on 2026-05-28.
