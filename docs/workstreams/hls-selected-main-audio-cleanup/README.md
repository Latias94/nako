# HLS Selected Main Audio Cleanup

Status: Completed
Last updated: 2026-05-29

This workstream removes the compatibility duplication left after HLS audio
sidecar artifacts shipped. Nako can now generate audio-only HLS sidecar
playlists and publish `TYPE=AUDIO` master playlist groups, but the selected
audio stream is still muxed into the primary video HLS output for
compatibility.

The lane shipped that cleanup. When a multi-audio HLS output advertises a
generated audio group, audio playback comes from that group instead of being
duplicated in each primary video variant. Single-audio and no-sidecar outputs
keep their current behavior.

Authoritative docs:

- `DESIGN.md`
- `TODO.md`
- `MILESTONES.md`
- `EVIDENCE_AND_GATES.md`
- `CLOSEOUT.md`
- `WORKSTREAM.json`
- `HANDOFF.md`
