# Audio Compatibility Downmix Normalization - Milestones

Status: Active
Last updated: 2026-05-30

## M0 - Scope And Evidence Freeze

Exit criteria:

- workstream files exist and agree on target state;
- architecture maps link the real workstream;
- `ACDN-020` is playback-only and has focused validation.

Status: Done.

## M1 - Playback Requirement Vocabulary

Exit criteria:

- `nako-playback` owns audio requirement values and compatibility reasons;
- channel support, downmix intent, and normalization intent can be expressed
  without invoking transcode command planning;
- focused playback tests pass.

Status: Ready.

## M2 - Transcode Policy Propagation

Exit criteria:

- transcode policy receives playback-owned audio requirements;
- HLS/remux adaptation remains explicit and testable;
- server playback code remains an adapter, not the source of audio policy.

Status: Pending `ACDN-020`.

## M3 - FFmpeg Audio Filter Planning

Exit criteria:

- command planning emits deterministic downmix/normalization filter decisions;
- direct/remux-compatible cases avoid unnecessary filters;
- HLS regression tests keep selected main audio and sidecar behavior stable.

Status: Pending `ACDN-030`.

## M4 - Diagnostics And Closeout

Exit criteria:

- evidence gates pass with fresh output;
- docs and `WORKSTREAM.json` reflect shipped behavior;
- UI preferences, device databases, and dialogue enhancement are split or
  deferred.

Status: Pending implementation.
