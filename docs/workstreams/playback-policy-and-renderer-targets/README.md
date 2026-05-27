# Playback Policy And Renderer Targets

Status: Active
Last updated: 2026-05-27

This workstream deepens Nako's playback policy boundary after the completed
Playback Transcode Policy lane. It turns "has Library Access Play" into an
explicit effective playback policy, introduces renderer target capability
records, and prepares the backend for browser, desktop, mobile, and future
casting without implementing casting protocols in this lane.

Authoritative docs:

- `DESIGN.md`
- `TODO.md`
- `MILESTONES.md`
- `EVIDENCE_AND_GATES.md`
- `HANDOFF.md`
- `WORKSTREAM.json`
- `docs/adr/0039-playback-policy-and-renderer-target-boundary.md`

First executable task: `PRT-020`, characterization tests for the current
Playback routes and planner proving that playback is only gated by Library
Access today and does not yet distinguish direct/remux/transcode/remote/cast
policy.
