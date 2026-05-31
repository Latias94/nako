# Transcode Capability Inventory Matrix - Milestones

Status: Active
Last updated: 2026-05-31

## M0 - Scope And Evidence Freeze

Exit criteria:

- workstream files exist and agree on inventory-only scope;
- architecture maps list the lane as safe parallel work;
- first executable task has focused `nako-transcode` gates.

Status: Done.

## M1 - Bitstream Filter Inventory Baseline

Exit criteria:

- capability inventory can represent optional bitstream-filter evidence;
- tests prove report construction without changing HLS runtime selection;
- no server or FFmpeg command execution changes are introduced.

Status: Done.

Evidence:

- static CPU and hardware capability reports expose optional
  `h264_mp4toannexb` bitstream-filter stage evidence;
- probe reports keep missing optional bitstream-filter evidence non-fatal for
  pipeline selection;
- validation remained inside `nako-transcode`.

## M2 - Broader Inventory Matrix Facts

Exit criteria:

- capability inventory can represent broader decoder, encoder, filter,
  tone-map, and subtitle evidence;
- tests prove report construction without changing HLS runtime selection;
- no server or FFmpeg command execution changes are introduced.

Status: Done.

Evidence:

- probe-derived reports expose optional HEVC/AV1 decoder facts;
- probe-derived reports expose optional future CPU and hardware encoder facts;
- probe-derived reports expose optional common, hardware, tone-map, and
  subtitle burn-in filter facts;
- missing broader facts remain non-fatal for pipeline selection;
- validation remained inside `nako-transcode`.

## M3 - Closeout

Exit criteria:

- fresh transcode capability gates pass;
- evidence ledger records the expanded inventory facts;
- policy/command/release follow-ons are split or deferred.

Status: Pending `TCIM-040`.
