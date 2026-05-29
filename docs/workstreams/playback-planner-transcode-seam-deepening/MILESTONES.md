# Playback Planner Transcode Seam Deepening - Milestones

Status: Completed
Last updated: 2026-05-29

## M0 - Open Lane

The workstream exists, is indexed, and has explicit gates.

## M1 - Builder Ownership Moved

`nako-transcode` owns playback profile request builders. `nako-playback` no
longer imports `TranscodeProfile`, `RemuxTranscodeProfile`,
`HlsTranscodeProfile`, `TranscodeExecutionPolicy`, or transcode validators.

## M2 - Server Composition Preserved

Playback app services construct remux/HLS profile identities through
`nako-transcode` and existing request identity tests continue to pass.

## M3 - Closeout

Evidence is recorded, focused gates pass, and remaining work is either closed
or split into follow-on lanes.

## Outcome

All milestones are complete. Feature work for seek/restart, HDR, audio
processing, subtitle burn-in, and runtime scheduling remains separate.
