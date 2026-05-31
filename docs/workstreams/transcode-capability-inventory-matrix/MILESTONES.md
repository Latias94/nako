# Transcode Capability Inventory Matrix - Milestones

Status: Active
Last updated: 2026-05-31

## M0 - Scope And Evidence Freeze

Exit criteria:

- workstream files exist and agree on inventory-only scope;
- architecture maps list the lane as safe parallel work;
- first executable task has focused `nako-transcode` gates.

Status: Done.

## M1 - Inventory Matrix Facts

Exit criteria:

- capability inventory can represent broader decoder, encoder, filter,
  tone-map, subtitle, and bitstream-filter evidence;
- tests prove report construction without changing HLS runtime selection;
- no server or FFmpeg command execution changes are introduced.

Status: Pending `TCIM-020`.

## M2 - Closeout

Exit criteria:

- fresh transcode capability gates pass;
- evidence ledger records the expanded inventory facts;
- policy/command/release follow-ons are split or deferred.

Status: Pending `TCIM-020`.
