# Playback Compatibility Matrix Hardening - Milestones

Status: Active
Last updated: 2026-05-31

## M0 - Scope And Evidence Freeze

Exit criteria:

- workstream files exist and agree on playback-only scope;
- architecture maps list the lane as safe parallel work;
- first executable task has focused `nako-playback` gates.

Status: Done.

## M1 - Playback Decision Matrix

Exit criteria:

- representative Direct Play, Remux, and HLS Transcode compatibility cases are
  expressed in a table-driven matrix;
- HDR tone-map-required cases prove Remux denial;
- audio output requirements prove downmix and normalization propagation;
- validation remains playback-only.

Status: Pending `PCMH-020`.

## M2 - Closeout

Exit criteria:

- fresh playback gates pass;
- evidence ledger records matrix coverage;
- follow-ons discovered by the matrix are split or deferred.

Status: Pending `PCMH-020`.
